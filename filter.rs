#![allow(dead_code)]

use super::db_driver::Join;
use std::fmt::{Display, Formatter};

pub trait Filterable {
    fn to_sql(&self) -> String;
}

pub enum Associativity {
    And,
    Or,
}

impl Display for Associativity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Associativity::And => write!(f, " AND "),
            Associativity::Or => write!(f, " OR "),
        }
    }
}

pub enum Filter {
    Users(UsersFilter),
    StudentAccount(StudentAccountFilter),
    TeacherAccount(TeacherAccountFilter),
    Courses(CoursesFilter),
    Departments(DepartmentsFilter),
    StudentCourses(StudentCoursesFilter),
}

impl Display for Filter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::Users(_) => write!(f, "USERS"),
            Filter::StudentAccount(_) => write!(f, "STUDENT_ACCOUNT"),
            Filter::TeacherAccount(_) => write!(f, "TEACHER_ACCOUNT"),
            Filter::Courses(_) => write!(f, "COURSES"),
            Filter::Departments(_) => write!(f, "DEPARTMENTS"),
            Filter::StudentCourses(_) => write!(f, "STUDENT_COURSES"),
        }
    }
}

impl Display for Join {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Join::Inner => write!(f, " INNER JOIN "),
            Join::Left => write!(f, " LEFT OUTER JOIN "),
            Join::Right => write!(f, " RIGHT OUTER JOIN "),
            Join::Full => write!(f, " FULL OUTER JOIN "),
        }
    }
}

impl Filterable for Filter {
    fn to_sql(&self) -> String {
        match self {
            Filter::Users(x) => x.to_sql(),
            Filter::StudentAccount(x) => x.to_sql(),
            Filter::TeacherAccount(x) => x.to_sql(),
            Filter::Courses(x) => x.to_sql(),
            Filter::Departments(x) => x.to_sql(),
            Filter::StudentCourses(x) => x.to_sql(),
        }
    }
}

pub enum UsersFilter {
    Username(String),
    Email(String),
    Phone(String),
    Role(String),
    Verified(bool),
    Suspended(bool),
    Forcenewpw(bool),
    Id(i32),
    All,
}

impl Filterable for UsersFilter {
    fn to_sql(&self) -> String {
        match self {
            UsersFilter::Username(username) => format!("username = '{}'", username),
            UsersFilter::Email(email) => format!("email = '{}'", email),
            UsersFilter::Phone(phone) => format!("phone = '{}'", phone),
            UsersFilter::Role(role) => format!("role = '{}'", role),
            UsersFilter::Verified(verified) => format!("verified = {}", verified),
            UsersFilter::Suspended(suspended) => format!("suspended = {}", suspended),
            UsersFilter::Forcenewpw(forcenewpw) => format!("forcenewpw = {}", forcenewpw),
            UsersFilter::Id(id) => format!("id = {}", id),
            UsersFilter::All => String::from("1 = 1"), // some condition that's always true
        }
    }
}

pub enum StudentAccountFilter {
    StudentId(i32),
    AdvisorId(i32),
    Discipline(String),
    Enrollment(String),
    Cgpa(f32),
    CanGrad(bool),
    CurCredit(i32),
    CumCredit(i32),
    Id(i32),
    All,
}

impl Filterable for StudentAccountFilter {
    fn to_sql(&self) -> String {
        match self {
            StudentAccountFilter::StudentId(student_id) => format!("student_id = {}", student_id),
            StudentAccountFilter::AdvisorId(advisor_id) => format!("advisor_id = {}", advisor_id),
            StudentAccountFilter::Discipline(discipline) => {
                format!("discipline = '{}'", discipline)
            }
            StudentAccountFilter::Enrollment(enrollment) => {
                format!("enrollment = '{}'", enrollment)
            }
            StudentAccountFilter::Cgpa(cgpa) => format!("cgpa = {}", cgpa),
            StudentAccountFilter::CanGrad(can_grad) => format!("can_grad = {}", can_grad),
            StudentAccountFilter::CurCredit(cur_credit) => format!("cur_credit = {}", cur_credit),
            StudentAccountFilter::CumCredit(cum_credit) => format!("cum_credit = {}", cum_credit),
            StudentAccountFilter::Id(id) => format!("id = {}", id),
            StudentAccountFilter::All => String::from("1 = 1"), // always true
        }
    }
}

pub enum TeacherAccountFilter {
    TeacherId(i32),
    DeptId(i32),
    Dept(String),
    Id(i32),
    All,
}

impl Filterable for TeacherAccountFilter {
    fn to_sql(&self) -> String {
        match self {
            TeacherAccountFilter::TeacherId(teacher_id) => format!("teacher_id = {}", teacher_id),
            TeacherAccountFilter::DeptId(dept_id) => format!("dept_id = {}", dept_id),
            TeacherAccountFilter::Dept(dept) => format!("dept = '{}'", dept),
            TeacherAccountFilter::Id(id) => format!("id = {}", id),
            TeacherAccountFilter::All => String::from("1 = 1"), // always true
        }
    }
}

pub enum CoursesFilter {
    Id(i32),
    TeacherId(i32),
    Course(String),
    CrCost(i32),
    CreatedAt(String),
    UpdatedAt(String),
    All,
}

impl Filterable for CoursesFilter {
    fn to_sql(&self) -> String {
        match self {
            CoursesFilter::Id(id) => format!("id = {}", id),
            CoursesFilter::TeacherId(teacher_id) => format!("teacher_id = {}", teacher_id),
            CoursesFilter::Course(course) => format!("course = '{}'", course),
            CoursesFilter::CrCost(cr_cost) => format!("cr_cost = {}", cr_cost),
            CoursesFilter::CreatedAt(created_at) => format!("created_at = '{}'", created_at),
            CoursesFilter::UpdatedAt(updated_at) => format!("updated_at = '{}'", updated_at),
            CoursesFilter::All => String::from("1 = 1"), // always true
        }
    }
}

pub enum DepartmentsFilter {
    DeptHead(i32),
    Name(String),
    Id(i32),
    All,
}

impl Filterable for DepartmentsFilter {
    fn to_sql(&self) -> String {
        match self {
            DepartmentsFilter::DeptHead(dept_head) => format!("dept_head = {}", dept_head),
            DepartmentsFilter::Name(name) => format!("name = '{}'", name),
            DepartmentsFilter::Id(id) => format!("id = {}", id),
            DepartmentsFilter::All => String::from("1 = 1"), // always true
        }
    }
}

pub enum StudentCoursesFilter {
    StudentId(i32),
    CourseId(i32),
    Grade(f64),
    Semester(String),
    Id(i32),
    All,
}

impl Filterable for StudentCoursesFilter {
    fn to_sql(&self) -> String {
        match self {
            StudentCoursesFilter::StudentId(student_id) => format!("student_id = {}", student_id),
            StudentCoursesFilter::CourseId(course_id) => format!("course_id = {}", course_id),
            StudentCoursesFilter::Grade(grade) => format!("grade = {}", grade),
            StudentCoursesFilter::Semester(semester) => format!("semester = '{}'", semester),
            StudentCoursesFilter::Id(id) => format!("id = {}", id),
            StudentCoursesFilter::All => String::from("1 = 1"), // always true
        }
    }
}
