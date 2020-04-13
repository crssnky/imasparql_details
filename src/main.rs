#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[get("/<subject>")]
fn get_data(subject: String) -> String {
  format!("You want to know {:?}", subject)
}

fn main() {
  rocket::ignite().mount("/", routes![get_data]).launch();
}
