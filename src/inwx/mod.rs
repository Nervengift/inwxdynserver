mod errors;

extern crate reqwest;
extern crate serde_xml_rs;

pub use self::errors::{UpdateError, UpdateResult};

const API_URL: &str = "https://api.domrobot.com/xmlrpc/";

// not a constant because those can't be used in fmt macros
macro_rules! api_xml {
    () => ("<?xml version=\"1.0\"?>
<methodCall>
   <methodName>nameserver.updateRecord</methodName>
   <params>
      <param>
         <value>
            <struct>
               <member>
                  <name>user</name>
                  <value>
                     <string>{user}</string>
                  </value>
               </member>
               <member>
                  <name>pass</name>
                  <value>
                     <string>{pass}</string>
                  </value>
               </member>
               <member>
                  <name>id</name>
                  <value>
                     <int>{domain_id}</int>
                  </value>
               </member>
               <member>
                  <name>content</name>
                  <value>
                     <string>{ip}</string>
                  </value>
               </member>
            </struct>
         </value>
      </param>
   </params>
</methodCall>")
}

#[derive(Deserialize, Debug)]
struct MethodResponse {
    params: Params,
}

#[derive(Deserialize, Debug)]
struct Params {
    param: Vec<Param>,
}

#[derive(Deserialize, Debug)]
struct Param {
    value: Value,
}

#[derive(Deserialize, Debug)]
enum Value {
    #[serde(rename = "struct")]
    Struct {member: Vec<Member>},
    #[serde(rename = "int")]
    Int(String),
    #[serde(rename = "double")]
    Double(String),
    #[serde(rename = "string")]
    String_(String),
}

#[derive(Deserialize, Debug)]
struct Member {
    name: Name,
    value: Value,
}

#[derive(Deserialize, Debug)]
struct Name {
    #[serde(rename = "$value")]
    name: String,
}

// example response
// <?xml version=\"1.0\" encoding=\"UTF-8\"?><methodResponse><params><param><value><struct><member><name>code</name><value><int>1000</int></value></member><member><name>msg</name><value><string>Command completed successfully</string></value></member><member><name>svTRID</name><value><string>20180120-136054940</string></value></member><member><name>runtime</name><value><double>0.705600</double></value></member></struct></value></param></params></methodResponse>
pub fn update_dns(user: &str, pass: &str, domain_id: u32, ip: &str) -> UpdateResult {
    let api_xml = format!(api_xml!(), user=user, pass=pass, domain_id=domain_id, ip=ip);

    let client = reqwest::Client::new();
    let mut res = client.post(API_URL).body(api_xml).send()?;

    let xml_resp = res.text().unwrap();
    let resp: MethodResponse = serde_xml_rs::deserialize(xml_resp.as_bytes()).unwrap();

    let s = match &resp.params.param[0].value {
        &Value::Struct{ref member} => member,
        _ => return Err(UpdateError::UnexpectedAnswer(xml_resp)),
    };
    let code = match &s.iter().find(|x| x.name.name == "code").unwrap().value {
        &Value::Int(ref code) => code.parse::<u32>().unwrap(),
        _ => return Err(UpdateError::UnexpectedAnswer(xml_resp)),
    };
    let msg = match &s.iter().find(|x| x.name.name == "msg").unwrap().value {
        &Value::String_(ref msg) => msg.clone(),
        _ => return Err(UpdateError::UnexpectedAnswer(xml_resp)),
    };
    if code != 1000 {
        return Err(UpdateError::UpdateFailed{code: code, message: msg});
    }
    Ok(())
}
