#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate percent_encoding;
extern crate serde_json;
extern crate ureq;
extern crate url;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use url::form_urlencoded;

#[derive(Debug, Deserialize)]
struct N {
  r#type: String,
  value: String,
}
#[derive(Debug, Deserialize)]
struct O {
  r#type: String,
  #[serde(default)]
  datatype: String,
  #[serde(default, rename = "xml:lang")]
  xml_lang: String,
  value: String,
}
#[derive(Debug, Deserialize)]
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

#[get("/<subject>")]
fn get_data(subject: String) -> String {
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
  let res = ureq::get(&base_url).call();
  println!("{}", base_url);

  if res.ok() {
    let json_str = res.into_string().unwrap();
    let res_json: Response = serde_json::from_str(&json_str).unwrap();
    if res_json.results.bindings.len() > 0 {
      return format!("You want to know {}\n{}", subject, json_str);
    }
  }
  format!("No result")
}

fn main() {
  rocket::ignite().mount("/", routes![get_data]).launch();
}
