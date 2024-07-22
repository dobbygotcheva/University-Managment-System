use anyhow::{Ok, Result};
use rusqlite::Connection;

pub struct DatabaseConnection {
    pub connection: Connection,
}

impl DatabaseConnection {
    pub fn new() -> Result<Self> {
        let connection = Connection::open("system.db")?;

        Ok(Self { connection })
    }

    pub fn create_tables(&mut self) -> Result<&mut Self> {
        self.connection.execute_batch(
                r#"
            BEGIN;
            CREATE TABLE IF NOT EXISTS "USERS" (
                "id" INTEGER NOT NULL UNIQUE,
                "username" TEXT NOT NULL,
                "password" TEXT NOT NULL,
                "email" TEXT NOT NULL UNIQUE,
                "phone" TEXT,
                "verified" BOOLEAN NOT NULL,
                "suspended" BOOLEAN NOT NULL,
                "forcenewpw" BOOLEAN NOT NULL,
                "role" TEXT NOT NULL,
                PRIMARY KEY("id" AUTOINCREMENT)
            );

            CREATE TABLE IF NOT EXISTS "STUDENT_ACCOUNT" (
                "id" INTEGER NOT NULL UNIQUE,
                "student_id" INTEGER NOT NULL UNIQUE,
                "advisor_id" INTEGER NOT NULL,
                "discipline" TEXT NOT NULL,
                "enrollment" TEXT NOT NULL,
                "cgpa" REAL NOT NULL,
                "can_grad" BOOLEAN NOT NULL,
                "cur_credit" INTEGER NOT NULL,
                "cum_credit" INTEGER NOT NULL,
                FOREIGN KEY ("student_id") REFERENCES "USERS"("id"),
                PRIMARY KEY("id" AUTOINCREMENT)
            );
            
            CREATE TABLE IF NOT EXISTS "TEACHER_ACCOUNT" (
                "id" INTEGER NOT NULL UNIQUE,
                "teacher_id" INTEGER NOT NULL UNIQUE,
                "dept_id" INTEGER NOT NULL,
                FOREIGN KEY ("teacher_id") REFERENCES "USERS"("id"),
                PRIMARY KEY("id" AUTOINCREMENT)
            );
            
            CREATE TABLE IF NOT EXISTS "COURSES" (
                "id" INTEGER NOT NULL UNIQUE,
                "teacher_id" INTEGER NOT NULL,
                "course" TEXT NOT NULL,
                "course_nr" TEXT NOT NULL,
                "description" TEXT,
                "cr_cost" INTEGER NOT NULL,
                "timeslots" TEXT NOT NULL,
                FOREIGN KEY ("teacher_id") REFERENCES "USERS"("id"),
                PRIMARY KEY("id" AUTOINCREMENT)
            );
            
            CREATE TABLE IF NOT EXISTS "STUDENT_COURSES" (
                "student_id" INTEGER NOT NULL,
                "course_id" INTEGER NOT NULL,
                "grade" REAL NOT NULL,
                "semester" TEXT NOT NULL,
                FOREIGN KEY ("student_id") REFERENCES "USERS"("id"),
                FOREIGN KEY ("course_id") REFERENCES "COURSES"("id")
            );
            
            CREATE TABLE IF NOT EXISTS "DEPARTMENTS" (
                "id" INTEGER NOT NULL,
                "name" TEXT NOT NULL,
                PRIMARY KEY("id" AUTOINCREMENT)
            );

            CREATE TRIGGER IF NOT EXISTS "manage_student_account_insert"
            AFTER INSERT ON "USERS"
            FOR EACH ROW
            WHEN NEW."role" = 'student'
            BEGIN
                INSERT OR REPLACE INTO "STUDENT_ACCOUNT" ("student_id", "advisor_id", "discipline", 
                "enrollment", "can_grad", "cgpa", "cur_credit", "cum_credit")
                VALUES (NEW.id, 0, '', '', FALSE, 0.0, 0, 0);
                DELETE FROM TEACHER_ACCOUNT WHERE "teacher_id" = NEW."id";
            END;

            CREATE TRIGGER IF NOT EXISTS "manage_student_account_update"
            AFTER UPDATE ON "USERS"
            FOR EACH ROW
            WHEN NEW."role" = 'student'
            BEGIN
                INSERT OR REPLACE INTO "STUDENT_ACCOUNT" ("student_id", "advisor_id", "discipline", 
                "enrollment", "can_grad", "cgpa", "cur_credit", "cum_credit")
                VALUES (NEW.id, 0, '', '', FALSE, 0.0, 0, 0);
                DELETE FROM TEACHER_ACCOUNT WHERE "teacher_id" = NEW."id";
            END;

            CREATE TRIGGER IF NOT EXISTS "manage_teacher_account_insert"
            AFTER INSERT ON "USERS"
            FOR EACH ROW
            WHEN NEW."role" = 'teacher'
            BEGIN
                INSERT OR REPLACE INTO "TEACHER_ACCOUNT" ("teacher_id", "dept_id")
                VALUES (NEW."id", 0);
                DELETE FROM STUDENT_ACCOUNT WHERE "student_id" = NEW."id";
            END;

            CREATE TRIGGER IF NOT EXISTS "manage_teacher_account_update"
            AFTER UPDATE ON "USERS"
            FOR EACH ROW
            WHEN NEW."role" = 'teacher'
            BEGIN
                INSERT OR REPLACE INTO "TEACHER_ACCOUNT" ("teacher_id", "dept_id")
                VALUES (NEW."id", 0);
                DELETE FROM STUDENT_ACCOUNT WHERE "student_id" = NEW."id";
            END;

            CREATE TRIGGER IF NOT EXISTS "clear_accounts_on_delete"
            AFTER DELETE ON "USERS"
            FOR EACH ROW
            BEGIN
                DELETE FROM STUDENT_ACCOUNT WHERE "student_id" = OLD."id";
                DELETE FROM TEACHER_ACCOUNT WHERE "teacher_id" = OLD."id";
            END;

            CREATE TRIGGER IF NOT EXISTS "handle_admin_role"
            AFTER INSERT ON USERS
            FOR EACH ROW
            WHEN NEW."role" = 'admin'
            BEGIN
                DELETE FROM STUDENT_ACCOUNT WHERE "student_id" = NEW."id";
                DELETE FROM TEACHER_ACCOUNT WHERE "teacher_id" = NEW."id";
            END;

            CREATE TRIGGER IF NOT EXISTS "update_student_cgpa_insert"
            AFTER INSERT ON "STUDENT_COURSES"
            FOR EACH ROW
            BEGIN
                UPDATE "STUDENT_ACCOUNT"
                SET "cgpa" = COALESCE((
                    SELECT SUM(CASE WHEN "grade" >= 0 THEN "grade" * "cr_cost" ELSE 0 END) / NULLIF(SUM(CASE WHEN "grade" >= 0 THEN "cr_cost" ELSE 0 END), 0)
                    FROM "STUDENT_COURSES"
                    JOIN "COURSES" ON "STUDENT_COURSES"."course_id" = "COURSES"."id"
                    WHERE "STUDENT_COURSES"."student_id" = NEW."student_id"
                ), 0.0),
                "can_grad" = CASE
                    WHEN COALESCE((SELECT SUM(CASE WHEN "grade" >= 0 THEN "cr_cost" ELSE 0 END) FROM "STUDENT_COURSES" WHERE "student_id" = NEW."student_id"), 0) >= 120 THEN 1
                    ELSE 0
                END
                WHERE "id" = NEW."student_id";
            END;

            CREATE TRIGGER IF NOT EXISTS "update_student_cgpa_update"
            AFTER UPDATE ON "STUDENT_COURSES"
            FOR EACH ROW
            BEGIN
                UPDATE "STUDENT_ACCOUNT"
                SET "cgpa" = COALESCE((
                    SELECT SUM(CASE WHEN "grade" >= 0 THEN "grade" * "cr_cost" ELSE 0 END) / NULLIF(SUM(CASE WHEN "grade" >= 0 THEN "cr_cost" ELSE 0 END), 0)
                    FROM "STUDENT_COURSES"
                    JOIN "COURSES" ON "STUDENT_COURSES"."course_id" = "COURSES"."id"
                    WHERE "STUDENT_COURSES"."student_id" = NEW."student_id"
                ), 0.0),
                "can_grad" = CASE
                    WHEN COALESCE((SELECT SUM(CASE WHEN "grade" >= 0 THEN "cr_cost" ELSE 0 END) FROM "STUDENT_COURSES" WHERE "student_id" = NEW."student_id"), 0) >= 120 THEN 1
                    ELSE 0
                END
                WHERE "id" = NEW."student_id";
            END;

            CREATE TRIGGER IF NOT EXISTS "update_student_cgpa_delete"
            AFTER DELETE ON "STUDENT_COURSES"
            FOR EACH ROW
            BEGIN
                UPDATE "STUDENT_ACCOUNT"
                SET "cgpa" = COALESCE((
                    SELECT SUM(CASE WHEN "grade" >= 0 THEN "grade" * "cr_cost" ELSE 0 END) / NULLIF(SUM(CASE WHEN "grade" >= 0 THEN "cr_cost" ELSE 0 END), 0)
                    FROM "STUDENT_COURSES"
                    JOIN "COURSES" ON "STUDENT_COURSES"."course_id" = COURSES."id"
                    WHERE "STUDENT_COURSES"."student_id" = OLD."student_id"
                ), 0.0),
                "can_grad" = CASE
                    WHEN COALESCE((SELECT SUM(CASE WHEN "grade" >= 0 THEN "cr_cost" ELSE 0 END) FROM "STUDENT_COURSES" WHERE "student_id" = OLD."student_id"), 0) >= 120 THEN 1
                    ELSE 0
                END
                WHERE "id" = OLD."student_id";
            END;

            COMMIT;
            "#,
            )?;

        Ok(self)
    }
}
