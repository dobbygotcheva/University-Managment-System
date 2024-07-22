#![allow(dead_code)]

use anyhow::Ok;
use anyhow::Result;
use rusqlite::types::ValueRef;
use std::cell::Cell;
use std::collections::HashMap;

use super::filter::*;
use super::sqlite_conn::*;
use super::table_models::*;

pub enum Join {
    Inner,
    Left,
    Right,
    Full,
}

#[derive(Debug)]
pub enum ReceiverType {
    User(User),
    StudentAccount(StudentAccount),
    TeacherAccount(TeacherAccount),
    Course(Courses),
    StudentCourse(StudentCourse),
    Department(Departments),
}

pub struct DbDriver {
    c: DatabaseConnection,
}

// Public methods for DbDriver
impl DbDriver {
    pub fn init() -> DbDriver {
        let mut c = DatabaseConnection::new().expect("Could not establish connection to database.");
        c.create_tables().expect("Could not create tables.");

        DbDriver { c }
    }

    pub fn find(
        &self,
        table: Table,
        filters: Vec<Filter>,
        join_mode: Option<Associativity>,
    ) -> Result<Vec<ReceiverType>> {
        let join_mode = join_mode.unwrap_or(Associativity::And);

        match table {
            Table::Users => {
                assert_eq!(
                    filters
                        .iter()
                        .map(|f| matches!(f, Filter::Users(_)))
                        .collect::<Vec<bool>>(),
                    filters.iter().map(|_| true).collect::<Vec<bool>>(),
                    "Invalid filter for table."
                );
                self.find_users(&filters, &join_mode)
            }

            Table::StudentAccount => {
                assert_eq!(
                    filters
                        .iter()
                        .map(|f| matches!(f, Filter::StudentAccount(_)))
                        .collect::<Vec<bool>>(),
                    filters.iter().map(|_| true).collect::<Vec<bool>>(),
                    "Invalid filter for table."
                );
                self.find_student_accounts(&filters, &join_mode)
            }

            Table::TeacherAccount => {
                assert_eq!(
                    filters
                        .iter()
                        .map(|f| matches!(f, Filter::TeacherAccount(_)))
                        .collect::<Vec<bool>>(),
                    filters.iter().map(|_| true).collect::<Vec<bool>>(),
                    "Invalid filter for table."
                );
                self.find_teacher_accounts(&filters, &join_mode)
            }

            Table::Courses => {
                assert_eq!(
                    filters
                        .iter()
                        .map(|f| matches!(f, Filter::Courses(_)))
                        .collect::<Vec<bool>>(),
                    filters.iter().map(|_| true).collect::<Vec<bool>>(),
                    "Invalid filter for table."
                );
                self.find_courses(&filters, &join_mode)
            }

            Table::StudentCourses => {
                assert_eq!(
                    filters
                        .iter()
                        .map(|f| matches!(f, Filter::StudentCourses(_)))
                        .collect::<Vec<bool>>(),
                    filters.iter().map(|_| true).collect::<Vec<bool>>(),
                    "Invalid filter for table."
                );
                self.find_student_courses(&filters, &join_mode)
            }

            Table::Departments => {
                assert_eq!(
                    filters
                        .iter()
                        .map(|f| matches!(f, Filter::Departments(_)))
                        .collect::<Vec<bool>>(),
                    filters.iter().map(|_| true).collect::<Vec<bool>>(),
                    "Invalid filter for table."
                );
                self.find_departments(&filters, join_mode)
            }
        }
    }

    pub fn insert(&mut self, data: Vec<ReceiverType>) -> Result<()> {
        for receiver in data.iter() {
            match receiver {
                ReceiverType::User(u) => self.insert_user(u)?,
                ReceiverType::StudentAccount(s) => self.insert_student_account(s)?,
                ReceiverType::TeacherAccount(t) => self.insert_teacher_account(t)?,
                ReceiverType::Course(c) => self.insert_course(c)?,
                ReceiverType::StudentCourse(s) => self.insert_student_course(s)?,
                ReceiverType::Department(d) => self.insert_department(d)?,
            }
        }

        Ok(())
    }

    pub fn update(&mut self, data: Vec<ReceiverType>) -> Result<()> {
        for receiver in data.iter() {
            match receiver {
                ReceiverType::User(u) => self.update_user(u)?,
                ReceiverType::StudentAccount(s) => self.update_student_account(s)?,
                ReceiverType::TeacherAccount(t) => self.update_teacher_account(t)?,
                ReceiverType::Course(c) => self.update_course(c)?,
                ReceiverType::StudentCourse(s) => self.update_student_course(s)?,
                ReceiverType::Department(d) => self.update_department(d)?,
            }
        }

        Ok(())
    }

    pub fn delete(&mut self, data: Vec<ReceiverType>) -> Result<()> {
        for receiver in data.iter() {
            match receiver {
                ReceiverType::User(u) => self.delete_user(u)?,
                ReceiverType::StudentAccount(s) => self.delete_student_account(s)?,
                ReceiverType::TeacherAccount(t) => self.delete_teacher_account(t)?,
                ReceiverType::Course(c) => self.delete_course(c)?,
                ReceiverType::StudentCourse(s) => self.delete_student_course(s)?,
                ReceiverType::Department(d) => self.delete_department(d)?,
            }
        }

        Ok(())
    }

    pub fn join_find(
        &mut self,
        tables: &[Table; 2],
        filters: Vec<Filter>,
        join: Join,
        assoc: Option<Associativity>,
    ) -> Result<Vec<HashMap<String, String>>> {
        let param = tables[0].join(&tables[1], join);
        let conditions = filters.iter().map(|f| f.to_sql()).collect::<Vec<String>>();
        let join_mode = assoc.unwrap_or(Associativity::And);
        let sql = format!(
            "SELECT * FROM {} WHERE {}",
            param,
            conditions.join(&join_mode.to_string())
        );

        let mut stmt = self.c.connection.prepare(&sql).unwrap();
        let mut stmt_cols = Cell::new(
            stmt.column_names()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
        );

        let rows = stmt.query_map([], |row| {
            let mut hm = HashMap::new();

            for (i, col) in stmt_cols.get_mut().iter().enumerate() {
                let value = match row.get_ref(i).unwrap_or_else(|_| ValueRef::Null) {
                    ValueRef::Null => "NULL".to_string(),
                    ValueRef::Integer(i) => i.to_string(),
                    ValueRef::Real(f) => f.to_string(),
                    ValueRef::Text(t) => String::from_utf8_lossy(t).to_string(),
                    ValueRef::Blob(b) => String::from_utf8_lossy(b).to_string(),
                };
                hm.insert(col.to_string(), value);
            }

            rusqlite::Result::Ok(hm)
        });

        let res: Vec<HashMap<String, String>> = rows
            .map(|row| row.map(|x| x.unwrap_or_default()))?
            .collect();

        Ok(res)
    }
}

// Private methods for DbDriver
impl DbDriver {
    fn delete_user(&mut self, data: &User) -> Result<()> {
        let sql = data.to_sql(Action::Delete);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn delete_student_account(&mut self, data: &StudentAccount) -> Result<()> {
        let sql = data.to_sql(Action::Delete);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn delete_teacher_account(&mut self, data: &TeacherAccount) -> Result<()> {
        let sql = data.to_sql(Action::Delete);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn delete_course(&mut self, data: &Courses) -> Result<()> {
        let sql = data.to_sql(Action::Delete);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn delete_student_course(&mut self, data: &StudentCourse) -> Result<()> {
        let sql = data.to_sql(Action::Delete);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn delete_department(&mut self, data: &Departments) -> Result<()> {
        let sql = data.to_sql(Action::Delete);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn update_user(&mut self, data: &User) -> Result<()> {
        let sql = data.to_sql(Action::Update);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn update_student_account(&mut self, data: &StudentAccount) -> Result<()> {
        let sql = data.to_sql(Action::Update);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn update_teacher_account(&mut self, data: &TeacherAccount) -> Result<()> {
        let sql = data.to_sql(Action::Update);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn update_course(&mut self, data: &Courses) -> Result<()> {
        let sql = data.to_sql(Action::Update);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn update_student_course(&mut self, data: &StudentCourse) -> Result<()> {
        let sql = data.to_sql(Action::Update);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn update_department(&mut self, data: &Departments) -> Result<()> {
        let sql = data.to_sql(Action::Update);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn insert_user(&mut self, data: &User) -> Result<()> {
        let sql = data.to_sql(Action::Insert);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn insert_student_account(&mut self, data: &StudentAccount) -> Result<()> {
        let sql = data.to_sql(Action::Insert);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn insert_teacher_account(&mut self, data: &TeacherAccount) -> Result<()> {
        let sql = data.to_sql(Action::Insert);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn insert_course(&mut self, data: &Courses) -> Result<()> {
        let sql = data.to_sql(Action::Insert);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn insert_student_course(&mut self, data: &StudentCourse) -> Result<()> {
        let sql = data.to_sql(Action::Insert);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn insert_department(&mut self, data: &Departments) -> Result<()> {
        let sql = data.to_sql(Action::Insert);
        self.c.connection.execute(&sql, [])?;

        Ok(())
    }

    fn find_departments(
        &self,
        filters: &[Filter],
        join_mode: Associativity,
    ) -> Result<Vec<ReceiverType>> {
        let sql = if filters.is_empty() {
            format!("SELECT * FROM DEPARTMENTS")
        } else {
            let conditions: Vec<String> = filters.iter().map(|f| f.to_sql()).collect();
            let separator = join_mode.to_string();
            format!(
                "SELECT * FROM DEPARTMENTS WHERE {}",
                conditions.join(&separator)
            )
        };

        let mut stmt = self.c.connection.prepare(&sql)?;
        let mut rows = stmt.query([])?;
        let mut departments = Vec::new();

        while let Some(row) = rows.next().unwrap_or(None) {
            let id: i32 = row.get(0)?;
            let name: String = row.get(1)?;

            departments.push(ReceiverType::Department(Departments {
                id,
                name,
            }))
        }

        Ok(departments)
    }

    fn find_student_courses(
        &self,
        filters: &[Filter],
        join_mode: &Associativity,
    ) -> Result<Vec<ReceiverType>> {
        let sql = if filters.is_empty() {
            format!("SELECT * FROM STUDENT_COURSES")
        } else {
            let conditions: Vec<String> = filters.iter().map(|f| f.to_sql()).collect();
            let separator = join_mode.to_string();
            format!(
                "SELECT * FROM STUDENT_COURSES WHERE {}",
                conditions.join(&separator)
            )
        };
        
        let mut stmt = self.c.connection.prepare(&sql)?;
        let mut rows = stmt.query([])?;
        let mut student_courses = Vec::new();

        while let Some(row) = rows.next().unwrap_or(None) {
            let student_id: i32 = row.get(0)?;
            let course_id: i32 = row.get(1)?;
            let grade: f32 = row.get(2)?;
            let semester: String = row.get(3)?;

            student_courses.push(ReceiverType::StudentCourse(StudentCourse {
                student_id,
                course_id,
                grade,
                semester,
            }))
        }

        Ok(student_courses)
    }

    fn find_courses(
        &self,
        filters: &[Filter],
        join_mode: &Associativity,
    ) -> Result<Vec<ReceiverType>> {
        let sql = if filters.is_empty() {
            format!("SELECT * FROM COURSES")
        } else {
            let conditions: Vec<String> = filters.iter().map(|f| f.to_sql()).collect();
            let separator = join_mode.to_string();
            format!(
                "SELECT * FROM COURSES WHERE {}",
                conditions.join(&separator)
            )
        };

        let mut stmt = self.c.connection.prepare(&sql)?;
        let mut rows = stmt.query([])?;
        let mut courses = Vec::new();

        while let Some(row) = rows.next().unwrap_or(None) {
            let id: i32 = row.get(0)?;
            let teacher_id: i32 = row.get(1)?;
            let course: String = row.get(2)?;
            let course_nr: String = row.get(3)?;
            let description: String = row.get(4)?;
            let cr_cost: i32 = row.get(5)?;
            let timeslots: String = row.get(6)?;

            courses.push(ReceiverType::Course(Courses {
                id,
                teacher_id,
                course,
                course_nr,
                description,
                cr_cost,
                timeslots,
            }))
        }

        Ok(courses)
    }

    fn find_teacher_accounts(
        &self,
        filters: &[Filter],
        join_mode: &Associativity,
    ) -> Result<Vec<ReceiverType>> {
        let sql = if filters.is_empty() {
            format!("SELECT * FROM TEACHER_ACCOUNT")
        } else {
            let conditions: Vec<String> = filters.iter().map(|f| f.to_sql()).collect();
            let separator = join_mode.to_string();
            format!(
                "SELECT * FROM TEACHER_ACCOUNT WHERE {}",
                conditions.join(&separator)
            )
        };

        let mut stmt = self.c.connection.prepare(&sql)?;
        let mut rows = stmt.query([])?;
        let mut teacher_accounts = Vec::new();

        while let Some(row) = rows.next().unwrap_or(None) {
            let id: i32 = row.get(0)?;
            let teacher_id: i32 = row.get(1)?;
            let dept_id: i32 = row.get(2)?;

            teacher_accounts.push(ReceiverType::TeacherAccount(TeacherAccount {
                id,
                teacher_id,
                dept_id
            }))
        }

        Ok(teacher_accounts)
    }

    fn find_student_accounts(
        &self,
        filters: &[Filter],
        join_mode: &Associativity,
    ) -> Result<Vec<ReceiverType>> {
        let sql = if filters.is_empty() {
            "SELECT * FROM STUDENT_ACCOUNT".to_owned()
        } else {
            let conditions: Vec<String> = filters.iter().map(|f| f.to_sql()).collect();
            let separator = join_mode.to_string();
            format!(
                "SELECT * FROM STUDENT_ACCOUNT WHERE {}",
                conditions.join(&separator)
            )
        };

        let mut stmt = self.c.connection.prepare(&sql)?;

        let mut rows = stmt.query([])?;
        let mut student_accounts = Vec::new();

        while let Some(row) = rows.next().unwrap_or(None) {
            let id: i32 = row.get(0)?;
            let student_id: i32 = row.get(1)?;
            let advisor_id: i32 = row.get(2)?;
            let discipline: String = row.get(3)?;
            let enrollment: String = row.get(4)?;
            let cgpa: f32 = row.get(5)?;
            let can_grad: bool = row.get(6)?;
            let cur_credit: i32 = row.get(7)?;
            let cum_credit: i32 = row.get(8)?;

            student_accounts.push(ReceiverType::StudentAccount(StudentAccount {
                id,
                student_id,
                advisor_id,
                discipline,
                enrollment,
                cgpa,
                can_grad,
                cur_credit,
                cum_credit,
            }))
        }

        Ok(student_accounts)
    }

    fn find_users(
        &self,
        filters: &[Filter],
        join_mode: &Associativity,
    ) -> Result<Vec<ReceiverType>> {
        let sql = if filters.is_empty() {
            format!("SELECT * FROM USERS")
        } else {
            let conditions: Vec<String> = filters.iter().map(|f| f.to_sql()).collect();
            let separator = join_mode.to_string();
            format!("SELECT * FROM USERS WHERE {}", conditions.join(&separator))
        };

        let mut stmt = self.c.connection.prepare(&sql)?;
        let mut rows = stmt.query([])?;
        let mut users = Vec::new();

        while let Some(row) = rows.next().unwrap_or(None) {
            let id: i32 = row.get(0)?;
            let username: String = row.get(1)?;
            let password: String = row.get(2)?;
            let email: String = row.get(3)?;
            let phone: String = row.get(4)?;
            let verified: bool = row.get(5)?;
            let suspended: bool = row.get(6)?;
            let forcenewpw: bool = row.get(7)?;
            let role: String = row.get(8)?;

            users.push(ReceiverType::User(User {
                id,
                username,
                password,
                email,
                phone,
                verified,
                suspended,
                forcenewpw,
                role,
            }))
        }

        Ok(users)
    }
}
