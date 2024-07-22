use actix_web::{delete, get, patch, post, HttpRequest, HttpResponse, Responder};
use serde_json::{json, Value};

use crate::backend::table_models::{User, TeacherAccount};
use crate::login_macro as login;

use super::{
    filter::{Filter, UsersFilter},
    server_connection_impl::*,
    table_models::Courses,
};

#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({"success": true}))
}

#[get("/users")]
pub async fn get_users() -> impl Responder {
    let conn = ServerConnection::new();
    let users = conn.get_users();
    match users {
        Ok(u) => {
            let json = serde_json::to_string(&u);
            match json {
                Ok(j) => HttpResponse::Ok().body(j),
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/students")]
pub async fn get_students() -> impl Responder {
    let conn = ServerConnection::new();
    let students = conn.get_users_by_filters(vec![Filter::Users(UsersFilter::Role(
        "student".to_string(),
    ))]);
    match students {
        Ok(s) => {
            let json = serde_json::to_string(&s);
            match json {
                Ok(j) => HttpResponse::Ok().body(j),
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/teachers")]
pub async fn get_teachers() -> impl Responder {
    let conn = ServerConnection::new();
    let teachers = conn.get_users_by_filters(vec![Filter::Users(UsersFilter::Role(
        "teacher".to_string(),
    ))]);
    match teachers {
        Ok(t) => {
            let json = serde_json::to_string(&t);
            match json {
                Ok(j) => HttpResponse::Ok().body(j),
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/departments")]
pub async fn get_departments() -> impl Responder {
    let conn = ServerConnection::new();
    let departments = conn.get_departments();
    match departments {
        Ok(d) => {
            let json = serde_json::to_string(&d);
            match json {
                Ok(j) => HttpResponse::Ok().body(j),
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/departments/{id}")]
pub async fn get_department(req: HttpRequest) -> impl Responder {
    let conn = ServerConnection::new();
    let request_headers = req.headers();
    let id = match request_headers.get("id") {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(json!({"error": "Missing department id."})),
    }
    .to_str()
    .unwrap();

    let id = match id.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({"error": "Invalid department id."}))
        }
    };

    let department = conn.get_department(id);
    match department {
        Ok(d) => {
            let json = serde_json::to_string(&d);
            match json {
                Ok(j) => HttpResponse::Ok().body(j),
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[post("/departments")]
pub async fn new_department(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let login_email = request_headers.get("login_email");
    let login_password = request_headers.get("login_password");
    login!(login_email, login_password, conn);
    if !conn.is_admin() {
        return HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
    }

    let department = request_headers.get("name").unwrap().to_str().unwrap();

    let department = conn.new_department(department);
    match department {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Successfully created department."})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[delete("/departments/{id}")]
pub async fn delete_department(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let login_email = request_headers.get("login_email");
    let login_password = request_headers.get("login_password");
    login!(login_email, login_password, conn);
    if !conn.is_admin() {
        return HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
    }

    let department = req.match_info().get("id").unwrap_or_else(|| "0");
    let department = department.parse::<i32>().unwrap_or_default();

    if department == 0 {
        return HttpResponse::BadRequest().json(json!({"error": "Missing department id."}));
    }

    let department = conn.get_department(department);
    let department = match department {
        Ok(d) => d,
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    let department = conn.remove_department(department);
    match department {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Successfully deleted department."})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[post("/admin/department/{id}")]
pub async fn invite_to_department(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let login_email = request_headers.get("login_email");
    let login_password = request_headers.get("login_password");
    login!(login_email, login_password, conn);
    if !conn.is_admin() {
        return HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
    }

    let department = req.match_info().get("id").unwrap();
    let department = department.parse::<i32>().unwrap_or_default();

    if department == 0 {
        return HttpResponse::BadRequest().json(json!({"error": "Missing department id."}));
    }

    let teacher = request_headers.get("teacher_id");

    if teacher.is_none() {
        return HttpResponse::BadRequest().json(json!({"error": "Missing teacher id."}));
    }

    let teacher = teacher.unwrap().to_str().unwrap();
    let teacher = teacher.parse::<i32>().unwrap_or_default();

    if teacher == 0 {
        return HttpResponse::BadRequest().json(json!({"error": "Invalid teacher id."}));
    }

    let department = conn.get_department(department);
    let department = match department {
        Ok(d) => d,
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    let teachers = conn.get_teacher_accounts();
    let mut teacher = match teachers {
        Ok(t) => match t
            .into_iter()
            .filter(|t| t.teacher_id == teacher)
            .collect::<Vec<_>>()
        {
            v if v.is_empty() => {
                return HttpResponse::BadRequest().json(json!({"error": "Teacher not found."}))
            }
            v => v[0].to_owned(),
        },
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    teacher.dept_id = department.id;

    let invitation = conn.update_teacher_account(teacher);

    match invitation {
        Ok(_) => HttpResponse::Ok()
            .json(json!({"message": "Successfully invited teacher to department."})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[delete("/admin/department/{id}")]
pub async fn kick_from_department(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let login_email = request_headers.get("login_email");
    let login_password = request_headers.get("login_password");
    login!(login_email, login_password, conn);
    if !conn.is_admin() {
        return HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
    }

    let department = req.match_info().get("id").unwrap();
    let department = department.parse::<i32>().unwrap_or_default();

    if department == 0 {
        return HttpResponse::BadRequest().json(json!({"error": "Missing department id."}));
    }

    let teacher = request_headers.get("teacher_id");
    if teacher.is_none() {
        return HttpResponse::BadRequest().json(json!({"error": "Missing teacher id."}));
    }

    let teacher = teacher.unwrap().to_str().unwrap();
    let teacher = teacher.parse::<i32>().unwrap_or_default();

    if teacher == 0 {
        return HttpResponse::BadRequest().json(json!({"error": "Invalid teacher id."}));
    }

    let teachers = conn.get_teacher_accounts();
    let mut teacher = match teachers {
        Ok(t) => match t
            .into_iter()
            .filter(|t| t.teacher_id == teacher)
            .collect::<Vec<_>>()
        {
            v if v.is_empty() => {
                return HttpResponse::BadRequest().json(json!({"error": "Teacher not found."}))
            }
            v => v[0].to_owned(),
        },
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    teacher.dept_id = 0;

    let invitation = conn.update_teacher_account(teacher);

    match invitation {
        Ok(_) => HttpResponse::Ok()
            .json(json!({"message": "Successfully kicked teacher from department."})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/courses")]
pub async fn get_courses() -> impl Responder {
    let conn = ServerConnection::new();

    let courses = match conn.search_courses("".to_string()) {
        Ok(c) => {
            let json = serde_json::to_string(&c);
            match json {
                Ok(j) => j,
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": e.to_string()}))
                }
            }
        }
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    let users = match conn.get_users_by_filters(vec![Filter::Users(UsersFilter::Role(
        "teacher".to_string(),
    ))]) {
        Ok(t) => {
            let json = serde_json::to_string(&t);
            match json {
                Ok(j) => j,
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": e.to_string()}))
                }
            }
        }
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    let departments = match conn.get_departments() {
        Ok(d) => {
            let json = serde_json::to_string(&d);
            match json {
                Ok(j) => j,
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": e.to_string()}))
                }
            }
        }
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    let teacher_accounts = match conn.get_teacher_accounts() {
        Ok(t) => {
            let json = serde_json::to_string(&t);
            match json {
                Ok(j) => j,
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": e.to_string()}))
                }
            }
        }
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    let json_prep = format!(
        "{{\"courses\": {}, \"users\": {}, \"teacher_accounts\": {}, \"departments\": {}}}",
        courses, users, teacher_accounts, departments
    );

    match serde_json::from_str::<Value>(&json_prep) {
        Ok(json3) => HttpResponse::Ok().json(json3),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/courses/{id}")]
pub async fn get_course(req: HttpRequest) -> impl Responder {
    let conn = ServerConnection::new();
    let id = req.match_info().get("id").unwrap_or_else(|| "0");

    if id == "0" {
        return HttpResponse::BadRequest().json(json!({"error": "Missing course id."}));
    }

    let course = match conn.search_courses(id.to_string()) {
        Ok(c) => c[0].to_owned(),
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    let user = match conn.get_users_by_filters(vec![Filter::Users(UsersFilter::Role(
        "teacher".to_string(),
    ))]) {
        Ok(t) => {
            let u = t
                .iter()
                .filter_map(|u| {
                    if course.teacher_id == u.id {
                        Some(u.to_owned())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            match u.get(0) {
                Some(u) => u.to_owned(),
                None => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "A teacher with this ID does not exist."}))
                }
            }
        }
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    let teacher_account = match conn.get_teacher_accounts() {
        Ok(t) => {
            let a = t
                .into_iter()
                .filter_map(|t| {
                    if course.teacher_id == t.teacher_id {
                        Some(t.to_owned())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            match a.get(0) {
                Some(a) => a.to_owned(),
                None => {
                    return HttpResponse::InternalServerError().json(
                        json!({"error": "A teacher account with this Teacher ID does not exist."}),
                    )
                }
            }
        }
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    let departments = match conn.get_departments() {
        Ok(d) => {
            let dep = d
                .into_iter()
                .filter_map(|d| {
                    if teacher_account.dept_id == d.id {
                        Some(d)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            match dep.get(0) {
                Some(d) => d.to_owned(),
                None => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "A department with such ID does not exist."}))
                }
            }
        }
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    let course = serde_json::to_string(&course).unwrap();
    let user = serde_json::to_string(&user).unwrap();
    let teacher_account = serde_json::to_string(&teacher_account).unwrap();
    let departments = serde_json::to_string(&departments).unwrap();

    let json_prep = format!(
        "{{\"course\": {}, \"user\": {}, \"teacher_account\": {}, \"department\": {}}}",
        course, user, teacher_account, departments
    );

    match serde_json::from_str::<Value>(&json_prep) {
        Ok(j) => HttpResponse::Ok().json(j),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[post("/courses")]
pub async fn new_course(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let login_email = request_headers.get("login_email");
    let login_password = request_headers.get("login_password");
    login!(login_email, login_password, conn);

    let course = request_headers.get("name");
    let description = request_headers.get("description");
    let course_nr = request_headers.get("course_nr");
    let teacher_id = request_headers.get("id");
    let cr_cost = request_headers.get("cr_cost");
    let timeslots = request_headers.get("timeslots");

    if course.is_none()
        || teacher_id.is_none()
        || course_nr.is_none()
        || cr_cost.is_none()
        || timeslots.is_none()
    {
        return HttpResponse::BadRequest().json(json!({"error": "Missing required data."}));
    }

    let course = course.unwrap().to_str().unwrap().to_string();
    let description = description
        .unwrap()
        .to_str()
        .unwrap_or("No description.")
        .to_string();
    let course_nr = course_nr.unwrap().to_str().unwrap().to_string();
    let teacher_id = teacher_id
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<i32>()
        .unwrap_or(0);
    let cr_cost = cr_cost
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<i32>()
        .unwrap_or(0);
    let timeslots = timeslots.unwrap().to_str().unwrap().to_string();

    if teacher_id == 0 {
        return HttpResponse::BadRequest().json(json!({"error": "Invalid teacher id."}));
    }

    if cr_cost == 0 {
        return HttpResponse::BadRequest().json(json!({"error": "Invalid course cost."}));
    }

    let course = Courses {
        id: 0, // This will be set by the database.
        description,
        teacher_id,
        course,
        course_nr,
        cr_cost,
        timeslots,
    };

    match conn.register_courses(vec![course]) {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Successfully registered course."})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[delete("/courses/{id}")]
pub async fn remove_course(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let id = req.match_info().get("id").unwrap();

    let login_email = request_headers.get("login_email");
    let login_password = request_headers.get("login_password");

    login!(login_email, login_password, conn);

    let find_course = conn.search_courses(id.to_string());

    match find_course {
        Ok(c) => {
            if c.len() == 0 {
                return HttpResponse::BadRequest().json(json!({"error": "Course not found."}));
            }

            if c.len() > 1 {
                return HttpResponse::InternalServerError()
                    .json(json!({"error": "Multiple courses found."}));
            }

            let course = c.get(0).unwrap().clone();

            match conn.remove_courses(vec![course]) {
                Ok(_) => {
                    HttpResponse::Ok().json(json!({"message": "Successfully removed course."}))
                }
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
            }
        }

        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
        }
    }
}

#[patch("/courses/{id}")]
pub async fn update_course(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let name = request_headers.get("name");
    let description = request_headers.get("description");
    let course_nr = request_headers.get("course_nr");
    let teacher_id = request_headers.get("id");
    let cr_cost = request_headers.get("cr_cost");
    let timeslots = request_headers.get("timeslots");

    let id = req.match_info().get("id").unwrap();

    if teacher_id.is_none()
        || cr_cost.is_none()
        || timeslots.is_none()
        || name.is_none()
        || course_nr.is_none()
    {
        return HttpResponse::BadRequest().json(json!({"error": "Missing data."}));
    }

    let teacher_id = teacher_id.unwrap().to_str().unwrap();
    let cr_cost = cr_cost.unwrap().to_str().unwrap();
    let timeslots = timeslots.unwrap().to_str().unwrap();
    let name = name.unwrap().to_str().unwrap();
    let course_nr = course_nr.unwrap().to_str().unwrap().to_string();
    let description = description
        .unwrap()
        .to_str()
        .unwrap_or("No description.")
        .to_string();

    let login_email = request_headers.get("login_email");
    let login_password = request_headers.get("login_password");

    login!(login_email, login_password, conn);

    let find_course = conn.search_courses(id.to_string());

    match find_course {
        Ok(c) => {
            if c.len() == 0 {
                return HttpResponse::BadRequest().json(json!({"error": "Course not found."}));
            }

            if c.len() > 1 {
                return HttpResponse::InternalServerError()
                    .json(json!({"error": "Multiple courses found."}));
            }

            let mut course = c.get(0).unwrap().clone();

            course.course = name.to_string();
            course.description = description;
            course.course_nr = course_nr;
            course.cr_cost = cr_cost.parse::<i32>().unwrap();
            course.timeslots = timeslots.to_string();
            course.teacher_id = teacher_id.parse::<i32>().unwrap();

            match conn.update_courses(vec![course]) {
                Ok(_) => {
                    HttpResponse::Ok().json(json!({"message": "Successfully updated course."}))
                }
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
            }
        }

        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
        }
    }
}

#[get("/admin")]
pub async fn admin(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let login_email = request_headers.get("login_email");
    let login_password = request_headers.get("login_password");

    login!(login_email, login_password, conn);
    if !conn.is_admin() {
        return HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
    }

    return HttpResponse::Ok().json(json!({"message": "Success"}));
}

#[patch("/admin/users/{id}")]
pub async fn update_user(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let login_email = request_headers.get("login_email");
    let login_password = request_headers.get("login_password");

    login!(login_email, login_password, conn);
    if !conn.is_admin() {
        return HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
    }

    let id = match req.match_info().get("id").unwrap().parse::<i32>() {
        Ok(i) => i,
        Err(_) => return HttpResponse::BadRequest().json(json!({"error": "Invalid id."})),
    };

    let mut lookup_user = match conn.search_users(format!("{}", id)) {
        Ok(u) => match u.get(0) {
            Some(u) => u.to_owned(),
            None => return HttpResponse::BadRequest().json(json!({"error": "User not found."})),
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
        }
    }
    .to_owned();

    let mut username = match request_headers.get("username") {
        Some(u) => u.to_str().unwrap().to_string(),
        None => String::new(),
    };
    let password = match request_headers.get("password") {
        Some(p) => p.to_str().unwrap().to_string(),
        None => String::new(),
    };
    let mut email = match request_headers.get("email") {
        Some(e) => e.to_str().unwrap().to_string(),
        None => String::new(),
    };
    let mut phone = match request_headers.get("phone") {
        Some(p) => p.to_str().unwrap().to_string(),
        None => String::new(),
    };
    let verified = match request_headers.get("verified") {
        Some(v) => match v.to_str().unwrap().parse::<bool>() {
            Ok(b) => b,
            Err(_) => {
                return HttpResponse::BadRequest().json(json!({"error": "Invalid verified."}))
            }
        },
        None => lookup_user.verified,
    };

    let suspended = match request_headers.get("suspended") {
        Some(s) => match s.to_str().unwrap().parse::<bool>() {
            Ok(b) => b,
            Err(_) => {
                return HttpResponse::BadRequest().json(json!({"error": "Invalid suspended."}))
            }
        },
        None => lookup_user.suspended,
    };

    let forcenewpw = match request_headers.get("forcenewpw") {
        Some(f) => match f.to_str().unwrap().parse::<bool>() {
            Ok(b) => b,
            Err(_) => {
                return HttpResponse::BadRequest().json(json!({"error": "Invalid forcenewpw."}))
            }
        },
        None => lookup_user.forcenewpw,
    };

    let mut role = match request_headers.get("role") {
        Some(r) => r.to_str().unwrap().to_string(),
        None => String::new(),
    };

    if username == "" {
        username = lookup_user.username;
    }
    if email == "" {
        email = lookup_user.email;
    }
    if phone == "" {
        phone = lookup_user.phone;
    }
    if role == "" {
        role = lookup_user.role;
    }

    lookup_user.username = username;
    lookup_user.password = password;
    lookup_user.email = email;
    lookup_user.phone = phone;
    lookup_user.verified = verified;
    lookup_user.suspended = suspended;
    lookup_user.forcenewpw = forcenewpw;
    lookup_user.role = role;

    match conn.update_user(lookup_user.clone()) {
        Ok(_) => {
            let json = serde_json::to_string(&lookup_user);
            match json {
                Ok(j) => return HttpResponse::Ok().body(j),
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": e.to_string()}))
                }
            }
        }
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[delete("/admin/users/{id}")]
pub async fn delete_user(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();

    let login_email = req.headers().get("login_email");
    let login_password = req.headers().get("login_password");

    let id = match req.match_info().get("id") {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json(json!({"error": "Invalid id."})),
    };

    login!(login_email, login_password, conn);

    let user = conn.search_users(format!("{}", id)).unwrap()[0].clone();

    match conn.delete_user(user) {
        Ok(_) => return HttpResponse::Ok().json(json!({"message": "Successfully deleted user."})),
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/account")]
pub async fn get_self(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let email = request_headers.get("login_email");
    let password = request_headers.get("login_password");

    login!(email, password, conn);

    let user_email = email.unwrap().to_str().unwrap();
    let user = match conn.search_users(format!("{}", user_email)) {
        Ok(users) => match users.get(0) {
            Some(u) => u.to_owned(),
            None => return HttpResponse::InternalServerError().json(json!({"error": "User not found."})),
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
        }
    };

    if conn.is_student() {
        let enrolled_in;

        let enrollments_json = match conn.list_enrollments() {
            Ok(e) => {
                enrolled_in = e;
                match serde_json::to_string(&enrolled_in) {
                    Ok(j) => j,
                    Err(e) => {
                        return HttpResponse::InternalServerError()
                            .json(json!({"error": e.to_string()}));
                    }
                }
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
            }
        };

        let courses_json = match conn.search_courses("".to_owned()) {
            Ok(c) => {
                let c = c
                    .into_iter()
                    .filter_map(|c| {
                        if enrolled_in.iter().any(|e| e.course_id == c.id) {
                            Some(c)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Courses>>();
                match serde_json::to_string(&c) {
                    Ok(j) => j,
                    Err(e) => {
                        return HttpResponse::InternalServerError()
                            .json(json!({"error": e.to_string()}));
                    }
                }
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
            }
        };

        let standing_json = match conn.get_student_standing() {
            Ok(s) => match serde_json::to_string(&s) {
                Ok(j) => j,
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": e.to_string()}));
                }
            },

            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
            }
        };

        let user_json = match serde_json::to_string(&user) {
            Ok(j) => j,
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
            }
        };

        let json_prep = format!(
            "{{\"user\": {}, \"enrollments\": {}, \"standing\": {}, \"courses\": {}}}",
            user_json, enrollments_json, standing_json, courses_json
        );

        match serde_json::from_str::<Value>(&json_prep) {
            Ok(json3) => HttpResponse::Ok().json(json3),
            Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
        }
    }

    else if conn.is_teacher() {
        let user_json = match serde_json::to_string(&user) {
            Ok(j) => j,
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
            }
        };

        let teacher_account;

        let teacher_accounts = match conn.get_teacher_accounts() {
            Ok(t) => {
                teacher_account = match t.into_iter().filter_map(|t| {
                    if t.teacher_id == user.id {
                        Some(t)
                    } else {
                        None
                    }})
                    .collect::<Vec<TeacherAccount>>().get(0) {
                        Some(t) => t.to_owned(),
                        None => {
                            return HttpResponse::InternalServerError().json(json!({"error": "A teacher account with this Teacher ID does not exist."}));
                        },
                    };

                match serde_json::to_string(&teacher_account) {
                    Ok(s) => s,
                    Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Failed to serialize teacher account."})),
                }
            }

            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
            }
        };

        let json_prep = format!("{{\"user\": {}, \"teacher_account\": {}}}", user_json, teacher_accounts);

        match serde_json::from_str::<Value>(&json_prep) {
            Ok(j) => HttpResponse::Ok().json(j),
            Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
        }
    }

    else {
        let user_json = match serde_json::to_string(&user) {
            Ok(j) => j,
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({"error": e.to_string()}));
            }
        };

        let json_prep = format!("{{\"user\": {}}}", user_json);

        match serde_json::from_str::<Value>(&json_prep) {
            Ok(json3) => HttpResponse::Ok().json(json3),
            Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
        }
    
    }
}

#[patch("/account")]
pub async fn update_self(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let login_email = request_headers.get("login_email");
    let login_password = request_headers.get("login_password");

    login!(login_email, login_password, conn);

    let mut user = conn
        .search_users(format!("{}", login_email.unwrap().to_str().unwrap()))
        .unwrap()[0]
        .to_owned();

    let username = request_headers.get("username");
    let email = request_headers.get("email");
    let password = request_headers.get("password");
    let phone = request_headers.get("phone");

    let mut username = match username {
        Some(u) => u.to_str().unwrap().to_string(),
        None => String::new(),
    };
    let mut email = match email {
        Some(e) => e.to_str().unwrap().to_string(),
        None => String::new(),
    };
    let password = match password {
        Some(p) => {
            user.forcenewpw = false;
            p.to_str().unwrap().to_string()
        }
        None => String::new(),
    };
    let mut phone = match phone {
        Some(p) => p.to_str().unwrap().to_string(),
        None => String::new(),
    };

    if username == "" {
        username = user.username;
    }
    if email == "" {
        email = user.email;
    }
    if phone == "" {
        phone = user.phone;
    }

    user.username = username;
    user.email = email;
    user.password = password;
    user.phone = phone;

    match conn.update_user(user.clone()) {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Successfully updated."})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[post("/enroll/{id}")]
pub async fn enroll(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let login_email = request_headers.get("login_email");
    let login_password = request_headers.get("login_password");

    login!(login_email, login_password, conn);

    let user = conn
        .search_users(format!("{}", login_email.unwrap().to_str().unwrap()))
        .unwrap()[0]
        .clone();

    let course_id = req.match_info().get("id");

    if course_id.is_none() {
        return HttpResponse::BadRequest().json(json!({"error": "Missing course id"}));
    }

    let course_id = course_id.unwrap().to_owned();

    match conn.enroll_courses(
        conn.search_courses(format!("{}", course_id))
            .unwrap()
            .iter()
            .filter_map(|c| Some(c.clone()))
            .collect(),
    ) {
        Ok(_) => {
            let json = serde_json::to_string(&user);
            match json {
                Ok(j) => return HttpResponse::Ok().json(j),
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Failed to serialize user"}))
                }
            }
        }

        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[post("/unenroll/{id}")]
pub async fn unenroll(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let email = request_headers.get("login_email");
    let password = request_headers.get("login_password");

    login!(email, password, conn);

    let user = conn
        .search_users(format!("{}", email.unwrap().to_str().unwrap()))
        .unwrap()[0]
        .to_owned();

    let course_id = match req.match_info().get("id") {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({"error": "Missing course id"}));
        }
    };

    let course_list = conn
        .search_courses(format!("{}", course_id))
        .unwrap()
        .iter()
        .filter_map(|c| Some(c.clone()))
        .collect::<Vec<Courses>>();

    match conn.drop_courses(course_list) {
        Ok(_) => {
            let json = serde_json::to_string(&user);
            match json {
                Ok(j) => return HttpResponse::Ok().body(j),
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Failed to serialize user"}))
                }
            }
        }

        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[post("/login")]
pub async fn login(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let email = request_headers.get("login_email");
    let password = request_headers.get("login_password");

    if email.is_none() || password.is_none() {
        return HttpResponse::BadRequest().json(json!({"error": "Missing username or password"}));
    }

    let email = email.unwrap().to_str().unwrap();
    let password = password.unwrap().to_str().unwrap();

    let user = conn.search_users(format!("{}", email));

    match user {
        Ok(u) => {
            if u.len() == 0 {
                return HttpResponse::BadRequest().json(json!({"error": "User not found"}));
            } else {
                let user = u.get(0).unwrap();

                match conn.login(email.to_owned(), password.to_owned()) {
                    Ok(_) => {
                        let json = serde_json::to_string(&user);
                        match json {
                            Ok(j) => return HttpResponse::Ok().body(j),
                            Err(e) => {
                                return HttpResponse::InternalServerError()
                                    .json(json!({"error": e.to_string()}))
                            }
                        }
                    }
                    Err(e) => {
                        return HttpResponse::InternalServerError()
                            .json(json!({"error": e.to_string()}))
                    }
                }
            }
        }
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/logout")]
pub async fn logout(req: HttpRequest) -> impl Responder {
    let request_headers = req.headers();

    let email = request_headers.get("login_email");
    let password = request_headers.get("login_password");

    if email.is_none() || password.is_none() {
        HttpResponse::Ok().json(json!({"message": "Successfully logged out."}))
    } else {
        HttpResponse::InternalServerError().json(json!({"error": "Failed to logout."}))
    }
}

#[post("/register")]
pub async fn register(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let username = request_headers.get("username");
    let password = request_headers.get("password");
    let email = request_headers.get("email");
    let phone = request_headers.get("phone");

    if username.is_none() || password.is_none() || email.is_none() {
        return HttpResponse::BadRequest()
            .json(json!({"error": "Missing username, password, email, or role"}));
    }

    let username = username.unwrap().to_str().unwrap().to_owned();
    let password = password.unwrap().to_str().unwrap().to_owned();
    let email = email.unwrap().to_str().unwrap().to_owned();
    let phone = phone
        .is_some_and(|_| true)
        .then(|| phone.unwrap().to_str().unwrap().to_owned())
        .or_else(|| Some(String::from("")))
        .unwrap_or_default();

    let u = User {
        id: 0,
        username,
        password,
        email,
        phone,
        verified: false,
        suspended: false,
        forcenewpw: false,
        role: String::from("student"),
    };

    match conn.register_user(u) {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Successfully registered."})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[post("/admin/register")]
pub async fn register_admin(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let username = request_headers.get("username");
    let password = request_headers.get("password");
    let email = request_headers.get("email");
    let phone = request_headers.get("phone");

    match request_headers.get("access_code") {
        Some(c) => {
            if c.to_str().unwrap() != "I_BECOME_THY_ADMIN_AND_I_FUCK_YOUR_MOTHER32131!@#@!#@!" {
                return HttpResponse::BadRequest().json(json!({"error": "Invalid access code."}));
            }
        }
        None => return HttpResponse::BadRequest().json(json!({"error": "Missing access code."})),
    };

    if username.is_none() || password.is_none() || email.is_none() {
        return HttpResponse::BadRequest()
            .json(json!({"error": "Missing username, password or email."}));
    }

    let username = username.unwrap().to_str().unwrap().to_owned();
    let password = password.unwrap().to_str().unwrap().to_owned();
    let email = email.unwrap().to_str().unwrap().to_owned();
    let phone = phone
        .is_some_and(|_| true)
        .then(|| phone.unwrap().to_str().unwrap().to_owned())
        .or_else(|| Some(String::from("")))
        .unwrap_or_default();

    let u = User {
        id: 0,
        username,
        password,
        email,
        phone,
        verified: true,
        suspended: false,
        forcenewpw: false,
        role: String::from("admin"),
    };

    match conn.register_user(u) {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Successfully registered."})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[get("/admin/stats")]
pub async fn get_stats(req: HttpRequest) -> impl Responder {
    let mut conn = ServerConnection::new();
    let request_headers = req.headers();

    let login_email = request_headers.get("login_email");
    let login_password = request_headers.get("login_password");
    login!(login_email, login_password, conn);

    if !conn.is_admin() {
        return HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
    }

    let stats = conn.generate_statistics();

    match stats {
        Ok(s) => {
            let json = serde_json::to_string(&s);
            match json {
                Ok(j) => HttpResponse::Ok().json(j),
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

// So that I don't have to repeat myself over and over again
#[macro_export]
macro_rules! login_macro {
    ($login_email:expr, $login_password:expr, $conn:expr) => {
        {
            match ($login_email, $login_password) {
                (Some(a), Some(b)) => {
                    let username = a.to_str().unwrap().to_owned();
                    let password = b.to_str().unwrap().to_owned();

                    match $conn.login(username, password) {
                        Ok(_) => {},
                        Err(_) => {
                            return HttpResponse::BadRequest().json(json!({"error": "Invalid login credentials."}));
                        }
                    }
                },
                (_, None) => {
                    return HttpResponse::BadRequest().json(json!({"error": "Missing login password."}));
                },
                (None, _) => {
                    return HttpResponse::BadRequest().json(json!({"error": "Missing login email."}));
                },
            }
        }
    }
}
