use super::db_driver::*;
use super::filter::*;
use super::password;
use super::table_models::*;

use anyhow::anyhow;
use anyhow::Ok;
use anyhow::Result;
use chrono::Datelike;
use regex::Regex;
use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Statistics {
    pub registered_users: i32,
    pub suspended_users: i32,
    pub faculty_members: i32,
    pub active_students: i32,
    pub graduated_students: i32,
    pub courses: i32,
    pub departments: i32,
}

pub struct ServerConnection {
    db: DbDriver,
    session: Option<User>,
}

// Public methods
impl ServerConnection {
    pub fn new() -> Self {
        Self {
            db: DbDriver::init(),
            session: None,
        }
    }

    // fetch all users from the database
    pub fn get_users(&self) -> Result<Vec<User>> {
        let users = self.db.find(Table::Users, vec![], None)?;
        let u = users
            .into_iter()
            .map(|x| {
                if let ReceiverType::User(user) = x {
                    user
                } else {
                    unreachable!()
                }
            })
            .collect();

        Ok(u)
    }

    pub fn get_users_by_filters(&self, filters: Vec<Filter>) -> Result<Vec<User>> {
        let finding = self.db.find(Table::Users, filters, None)?;

        let users = finding
            .into_iter()
            .filter_map(|x| {
                if let ReceiverType::User(user) = x {
                    Some(user)
                } else {
                    None
                }
            })
            .collect();

        Ok(users)
    }

    pub fn register_user(&mut self, user: User) -> Result<()> {
        if !self.session.is_none() {
            return Err(anyhow!("Must be signed out."));
        }

        let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@aubg\.edu$")?;
        let phone_regex = Regex::new(r#"^\+?[0-9]{2}[-. ]?[0-9]{4}[-. ]?[0-9]{4}$"#)?;
        let password_rules = user.password.len() >= 8
            && user.password.chars().any(|c| c.is_ascii_lowercase())
            && user.password.chars().any(|c| c.is_ascii_uppercase())
            && user.password.chars().any(|c| c.is_ascii_digit())
            && user.password.chars().any(|c| "@$!%*?&".contains(c));

        if self
            .get_users_by_filters(vec![Filter::Users(UsersFilter::Email(
                user.email.to_lowercase().clone(),
            ))])?
            .len()
            > 0
        {
            return Err(anyhow!("A user with this email already exists."));
        }

        if user.username.is_empty() {
            return Err(anyhow!("Account name cannot be empty."));
        }

        if !email_regex.is_match(&user.email) {
            return Err(anyhow!("Must be a valid AUBG email."));
        }

        if !phone_regex.is_match(&user.phone) && !user.phone.is_empty() {
            return Err(anyhow!("Invalid phone number."));
        }

        if !password_rules {
            return Err(anyhow!(
                "The password does not meet the following criteria:\n
            - Must be at least 8 characters long\n
            - Must contain at least 1 uppercase letter\n
            - Must contain at least 1 lowercase letter\n
            - Must contain at least 1 number\n
            - Must contain at least 1 special character (@, $, !, %, *, ?, &)\n"
            ));
        }

        let mut user = user.to_owned();

        let salt = password::generate_salt();
        user.password = password::hash(&user.password, salt);

        self.db.insert(vec![ReceiverType::User(user)])?;

        Ok(())
    }

    pub fn login(&mut self, email: String, password: String) -> Result<()> {
        let binding = self.get_users_by_filters(vec![Filter::Users(UsersFilter::Email(email))])?;
        let user = binding.get(0).ok_or_else(|| anyhow!("User not found."))?; // if none, user not found

        // If the user is suspended, they cannot login
        if user.suspended {
            return Err(anyhow!("User is suspended."));
        }

        if user.forcenewpw {
            return Err(anyhow!("User must change password."));
        }

        // check hash for validity and then compare both server and client password hashes
        if password::verify(&user.password, &password) {
            self.session = Some(user.to_owned());
            Ok(())
        } else {
            Err(anyhow!("Invalid username or password."))
        }
    }

    pub fn update_user(&mut self, user: User) -> Result<()> {
        if let Some(s) = &self.session {
            match s.role.to_lowercase().as_str() {
                "admin" => self.update_user_as_admin(user)?,
                _ => self.update_user_as_student(user)?,
            }
        } else {
            return Err(anyhow!("Must be signed in."));
        }
        Ok(())
    }

    pub fn delete_user(&mut self, user: User) -> Result<()> {
        if let Some(s) = &self.session {
            match s.role.to_lowercase().as_str() {
                "admin" => {
                    if user.id != s.id {
                        self.db.delete(vec![ReceiverType::User(user.clone())])?;

                        Ok(())
                    } else {
                        Err(anyhow!(
                            "You cannot delete your own account as an administrator."
                        ))
                    }
                }
                _ => {
                    if user.id == s.id {
                        self.db.delete(vec![ReceiverType::User(user.clone())])?;

                        Ok(())
                    } else {
                        Err(anyhow!("You do not have permission to delete this user."))
                    }
                }
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn register_courses(&mut self, courses: Vec<Courses>) -> Result<()> {
        if let Some(s) = &self.session {
            match s.role.to_lowercase().as_str() {
                "admin" => {
                    let upcast = courses
                        .iter()
                        .map(|x| ReceiverType::Course(x.to_owned()))
                        .collect();

                    self.db.insert(upcast)?;

                    Ok(())
                }
                "teacher" => {
                    let errors = courses
                        .iter()
                        .filter_map(|x| {
                            if x.teacher_id != s.id {
                                Some(
                                    anyhow!(
                                        "You do not have permission to register courses on someone else's behalf."
                                    )
                                )
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    if errors.len() > 0 {
                        return Err(
                            anyhow!(
                                "You do not have permission to register courses on someone else's behalf. No action was taken."
                            )
                        );
                    }

                    let upcast = courses
                        .iter()
                        .map(|x| ReceiverType::Course(x.to_owned()))
                        .collect();

                    self.db.insert(upcast)?;

                    Ok(())
                }
                _ => Err(anyhow!("You do not have permission to register courses.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn remove_courses(&mut self, courses: Vec<Courses>) -> Result<()> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "admin" => {
                    let upcast = courses
                        .iter()
                        .map(|x| ReceiverType::Course(x.to_owned()))
                        .collect();

                    self.db.delete(upcast)?;

                    Ok(())
                }
                "teacher" => {
                    let errors = courses
                        .iter()
                        .filter_map(|x| {
                            if x.teacher_id != session.id {
                                Some(anyhow!("You do not have permission to remove this course."))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    if errors.len() > 0 {
                        return Err(anyhow!(
                            "Some courses to not belong to you. No action was taken."
                        ));
                    }

                    let upcast = courses
                        .iter()
                        .map(|x| ReceiverType::Course(x.to_owned()))
                        .collect();

                    self.db.delete(upcast)?;

                    Ok(())
                }
                _ => Err(anyhow!("You do not have permission to remove courses.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn update_courses(&mut self, courses: Vec<Courses>) -> Result<()> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "admin" => {
                    let upcast = courses
                        .iter()
                        .map(|x| ReceiverType::Course(x.to_owned()))
                        .collect();

                    self.db.update(upcast)?;

                    Ok(())
                }
                "teacher" => {
                    let errors = courses
                        .iter()
                        .filter_map(|x| {
                            if x.teacher_id != session.id {
                                Some(anyhow!("You do not have permission to update this course."))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    if errors.len() > 0 {
                        return Err(anyhow!(
                            "Some courses to not belong to you. No action was taken."
                        ));
                    }

                    let upcast = courses
                        .iter()
                        .map(|x| ReceiverType::Course(x.to_owned()))
                        .collect();

                    self.db.update(upcast)?;

                    Ok(())
                }
                _ => Err(anyhow!("You do not have permission to update courses.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn search_users(&self, query: String) -> Result<Vec<User>> {
        let findings = self.db.find(
            Table::Users,
            vec![],
            None,
        )?;

        let users = findings
            .into_iter()
            .filter_map(|x| {
                if let ReceiverType::User(user) = x {
                    Some(user)
                } else {
                    None
                }
            })
            .filter(|x| {
                    x.email.contains(&query)
                    || x.id.to_string().contains(&query)
            })
            .collect();

        Ok(users)
    }

    pub fn search_courses(&self, query: String) -> Result<Vec<Courses>> {
        let findings = self.db.find(
            Table::Courses,
            vec![],
            None,
        )?;

        let courses = findings
            .into_iter()
            .filter_map(|x| {
                if let ReceiverType::Course(course) = x {
                    Some(course)
                } else {
                    None
                }
            })
            .filter(|x| {
                x.id.to_string().contains(&query)
            })
            .collect();

        Ok(courses)
    }

    pub fn get_departments(&self) -> Result<Vec<Departments>> {
        let findings = self.db.find(
            Table::Departments,
            vec![],
            None,
        )?;

        let departments = findings
            .into_iter()
            .filter_map(|x| {
                if let ReceiverType::Department(department) = x {
                    Some(department)
                } else {
                    None
                }
            })
            .collect();

        Ok(departments)
    }

    pub fn get_department(&self, id: i32) -> Result<Departments> {
        let findings = self.db.find(
            Table::Departments,
            vec![Filter::Departments(DepartmentsFilter::Id(id))],
            None,
        )?;

        let departments: Vec<Departments> = findings
            .into_iter()
            .filter_map(|x| {
                if let ReceiverType::Department(department) = x {
                    Some(department)
                } else {
                    None
                }
            })
            .collect();

        let department = departments.get(0).ok_or_else(|| anyhow!("Department not found."))?;

        Ok(department.to_owned())
    }

    pub fn new_department(&mut self, department: &str) -> Result<()> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "admin" => {
                    let department = Departments {
                        id: 0,
                        name: department.to_owned(),
                    };
                    self.db.insert(vec![ReceiverType::Department(department)])?;

                    Ok(())
                }
                _ => Err(anyhow!("Only admins can create departments.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn remove_department(&mut self, department: Departments) -> Result<()> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "admin" => {
                    self.db.delete(vec![ReceiverType::Department(department)])?;

                    Ok(())
                }
                _ => Err(anyhow!("Only admins can remove departments.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn get_teacher_accounts(&self) -> Result<Vec<TeacherAccount>> {
        let findings = self.db.find(
            Table::TeacherAccount,
            vec![],
            None,
        )?;

        let teacher_accounts = findings
            .into_iter()
            .filter_map(|x| {
                if let ReceiverType::TeacherAccount(teacher_account) = x {
                    Some(teacher_account)
                } else {
                    None
                }
            })
            .collect();

        Ok(teacher_accounts)
    }

    pub fn update_teacher_account(&mut self, teacher_account: TeacherAccount) -> Result<()> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "admin" => {
                    self.db.update(vec![ReceiverType::TeacherAccount(teacher_account)])?;
                    Ok(())
                }
                _ => Err(anyhow!("Only admins can update teacher accounts.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn enroll_courses(&mut self, courses: Vec<Courses>) -> Result<()> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "student" => {
                    let errors = courses
                        .iter()
                        .filter_map(|x| {
                            let c = self.transmute_course_to_student_course(x.to_owned());
                            if c.student_id != session.id {
                                Some(
                                    anyhow!(
                                        "You do not have permission to register courses on someone else's behalf."
                                    )
                                )
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    if errors.len() > 0 {
                        return Err(
                            anyhow!(
                                "You do not have permission to register courses on someone else's behalf. No action was taken."
                            )
                        );
                    }

                    let upcast = courses
                        .iter()
                        .map(|x| {
                            ReceiverType::StudentCourse(
                                self.transmute_course_to_student_course(x.to_owned()),
                            )
                        })
                        .collect();

                    self.db.insert(upcast)?;

                    Ok(())
                }
                _ => Err(anyhow!("You do not have permission to enroll courses.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn list_enrollments(&self) -> Result<Vec<StudentCourse>> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "student" => {
                    let findings = self.db.find(
                        Table::StudentCourses,
                        vec![Filter::StudentCourses(StudentCoursesFilter::StudentId(session.id))],
                        None,
                    )?;

                    let courses = findings
                        .into_iter()
                        .filter_map(|x| {
                            if let ReceiverType::StudentCourse(course) = x {
                                Some(course)
                            } else {
                                None
                            }
                        })
                        .collect();

                    Ok(courses)
                }
                _ => Err(anyhow!("You are not a student.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn get_student_standing(&self) -> Result<StudentAccount> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "student" => {
                    let findings = self.db.find(
                        Table::StudentAccount,
                        vec![Filter::StudentAccount(StudentAccountFilter::StudentId(
                        session.id
                        ))],
                        None
                    );

                    let student = findings?
                        .into_iter()
                        .filter_map(|x| {
                            if let ReceiverType::StudentAccount(student) = x {
                                Some(student)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    Ok(student[0].to_owned())
                }
                _ => Err(anyhow!("You are not a student.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn drop_courses(&mut self, courses: Vec<Courses>) -> Result<()> {
        if let Some(session) = &self.session {
            match session.role.to_lowercase().as_str() {
                "student" => {
                    let errors = courses
                        .iter()
                        .filter_map(|x| {
                            let c = self.transmute_course_to_student_course(x.to_owned());
                            if c.student_id != session.id {
                                Some(
                                    anyhow!(
                                        "You do not have permission to register courses on someone else's behalf."
                                    )
                                )
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    if errors.len() > 0 {
                        return Err(
                            anyhow!(
                                "You do not have permission to register courses on someone else's behalf. No action was taken."
                            )
                        );
                    }

                    let upcast = courses
                        .iter()
                        .map(|x| {
                            ReceiverType::StudentCourse(
                                self.transmute_course_to_student_course(x.to_owned()),
                            )
                        })
                        .collect();

                    self.db.delete(upcast)?;

                    Ok(())
                }
                _ => Err(anyhow!("You do not have permission to drop courses.")),
            }
        } else {
            Err(anyhow!("Must be signed in."))
        }
    }

    pub fn is_student(&self) -> bool {
        if let Some(session) = &self.session {
            session.role.to_lowercase() == "student"
        } else {
            false
        }
    }

    pub fn is_teacher(&self) -> bool {
        if let Some(session) = &self.session {
            session.role.to_lowercase() == "teacher"
        } else {
            false
        }
    }

    pub fn is_admin(&self) -> bool {
        if let Some(session) = &self.session {
            session.role.to_lowercase() == "admin"
        } else {
            false
        }
    }

    pub fn generate_statistics(&self) -> Result<Statistics> {
        let registered_users = self.get_users()?.len() as i32;
        let suspended_users = self
            .get_users_by_filters(vec![Filter::Users(UsersFilter::Suspended(true))])?
            .len() as i32;
        let faculty_members = self
            .get_users_by_filters(vec![Filter::Users(UsersFilter::Role(
                "teacher".to_string(),
            ))])?
            .len() as i32;
        let active_students = self
            .get_users_by_filters(vec![Filter::Users(UsersFilter::Role(
                "student".to_string(),
            ))])?
            .len() as i32
            - suspended_users;
        let graduated_students = self
            .get_users_by_filters(vec![Filter::StudentAccount(StudentAccountFilter::CanGrad(
                true,
            ))])?
            .len() as i32;
        let courses = self.db.find(Table::Courses, vec![], None)?.len() as i32;
        let departments = self.db.find(Table::Departments, vec![], None)?.len() as i32;

        Ok(Statistics {
            registered_users,
            suspended_users,
            faculty_members,
            active_students,
            graduated_students,
            courses,
            departments,
        })
    }
}

// Private methods
impl ServerConnection {
    fn transmute_course_to_student_course(&self, course: Courses) -> StudentCourse {
        StudentCourse {
            student_id: self.session.as_ref().unwrap().id,
            course_id: course.id,
            grade: -1.0,
            semester: match chrono::Local::now().month() {
                6..=12 => "Fall".to_string(),
                _ => "Spring".to_string(),
            },
        }
    }

    fn update_user_as_student(&mut self, mut user: User) -> Result<()> {
        let binding =
            self.get_users_by_filters(vec![Filter::Users(UsersFilter::Id(user.id.clone()))])?;
        let u = binding.get(0).ok_or_else(|| anyhow!("User not found."))?;

        // Check permissions
        if user.suspended != u.suspended {
            return Err(anyhow!("Suspended cannot be changed."));
        }

        if user.verified != u.verified {
            return Err(anyhow!("Verified cannot be changed."));
        }

        if user.role != u.role {
            return Err(anyhow!("Role cannot be changed."));
        }
        
        if user.password.is_empty() || user.password.starts_with("$argon2id") {
            user.password = u.password.to_owned();
        } else {
            let salt = password::generate_salt();
            user.password = password::hash(&user.password, salt);
        }

        self.db.update(vec![ReceiverType::User(user)])?;

        Ok(())
    }

    fn update_user_as_admin(&mut self, mut user: User) -> Result<()> {
        let binding =
            self.get_users_by_filters(vec![Filter::Users(UsersFilter::Id(user.id.clone()))])?;
        let u = binding.get(0).ok_or_else(|| anyhow!("User not found."))?;

        if user.password.is_empty() || user.password.starts_with("$argon2id") {
            user.password = u.password.to_owned();
        } else {
            let salt = password::generate_salt();
            user.password = password::hash(&user.password, salt);
        }

        self.db.update(vec![ReceiverType::User(user)])?;

        Ok(())
    }
}
