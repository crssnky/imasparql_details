#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate percent_encoding;
extern crate serde_json;
extern crate ureq;
extern crate url;
use percent_encoding::{percent_decode, utf8_percent_encode, AsciiSet, CONTROLS};
use rocket::Request;
use rocket_contrib::templates::Template;
use url::form_urlencoded;

#[derive(Debug, Deserialize, Serialize)]
struct N {
  r#type: String,
  value: String,
}
#[derive(Debug, Deserialize, Serialize)]
struct O {
  r#type: String,
  #[serde(default)]
  datatype: String,
  #[serde(default, rename = "xml:lang")]
  xml_lang: String,
  value: String,
}
#[derive(Debug, Deserialize, Serialize)]
struct Bindings {
  n: N,
  o: O,
}
#[derive(Debug, Deserialize)]
struct Results {
  bindings: Vec<Bindings>,
}
#[derive(Debug, Deserialize)]
struct Head {
  vars: Vec<String>,
}
#[derive(Debug, Deserialize)]
struct Response {
  head: Head,
  results: Results,
}
#[derive(Serialize)]
struct MessageContent {
  title: String,
  num: usize,
  json: Vec<Bindings>,
}

#[catch(404)]
fn not_found(_req: &Request) -> String {
  format!("No result: ")
}

#[get("/<subject>")]
fn get_data(subject: String) -> Template {
  const FRAGMENT: &AsciiSet = &CONTROLS;
  let encoded_subject = utf8_percent_encode(&subject, FRAGMENT).to_string();
  let quety = format!("PREFIX schema: <http://schema.org/>PREFIX imas: <https://sparql.crssnky.xyz/imasrdf/RDFs/detail/>SELECT * WHERE {{ imas:{} ?n ?o;}}order by (?n)", encoded_subject);
  let encoded_query = form_urlencoded::Serializer::new(String::new())
    .append_pair("output", "json")
    .append_pair("force-accept", "text/plain")
    .append_pair("query", &quety)
    .finish();
  let base_url = format!(
    "https://sparql.crssnky.xyz/spql/imas/query?{}",
    encoded_query
  );
  println!("encoded: {}", &encoded_subject);
  let res = ureq::get(&base_url).call();

  if res.ok() {
    let json_str = res.into_string().unwrap();
    let res_json: Response = serde_json::from_str(&json_str).unwrap();
    let mut json = res_json.results.bindings;
    let json_num = json.len();
    if json_num > 0 {
      for data in &mut json {
        match &*(data.n.value) {
          "http://schema.org/memberOf" => {
            data.o.value = percent_decode(data.o.value.as_bytes())
              .decode_utf8()
              .unwrap()
              .to_string();
          }
          "http://schema.org/owns" => {
            data.o.value = percent_decode(data.o.value.as_bytes())
              .decode_utf8()
              .unwrap()
              .to_string();
          }
          _ => (),
        }
      }
      let content = MessageContent {
        title: subject,
        num: json_num,
        json: json,
      };
      return Template::render("detail", &content);
    }
  }
  let content = MessageContent {
    title: subject,
    num: 0,
    json: vec![],
  };
  Template::render("error", &content)
}

fn rocket() -> rocket::Rocket {
  rocket::ignite()
    .mount("/", routes![get_data])
    .attach(Template::fairing())
    .register(catchers![not_found])
}

fn main() {
  rocket().launch();
}
