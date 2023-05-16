use std::{io};
use actix_files::{Files, NamedFile};
use serde::{Deserialize};
use askama::Template;
use std::path::{PathBuf};
use actix_web::{
    get,
    http::{
        header::{self, ContentType},
        Method, StatusCode,
    },
    web, App, Either, HttpRequest, HttpServer, Responder, Result,ResponseError
};
use mysql::params;

////////////////use de thomas/////////////////
use mysql::*;
use mysql::prelude::*;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::hash::{Hash, Hasher};
use actix_web::HttpResponse;
use std::fmt::Debug;
use rand::distributions::DistString;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce // Or `Aes128Gcm`
};
use argon2::{self, Config};
use generic_array::{GenericArray, sequence::GenericSequence};


//Dépendances pour le mail
use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;
////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq)]
struct Password {
    sel_1: String,
    sel_2: String,
    sel_gcm: String,
    clefs: String,
    login: String,
    passw: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Salo {
	labaleine: String,
}



//fonction de hash
impl Hash for Password {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.passw.hash(state);
    }
}



fn salt() -> String {
let mut rng = thread_rng();
let _x: u32 = rng.gen();


let s: String = (&mut rng).sample_iter(Alphanumeric)
    .take(15)
    .map(char::from)
    .collect();
return s;
}



fn passwordhash(a:String ,b: String) -> String {
let password = b.as_bytes();
let salt = a.as_bytes();
let config = Config::default();
let encoded_hash = argon2::hash_encoded(password, salt, &config).unwrap();
let hash = encoded_hash.as_str();
println!("ceci est le hash argon2: {}", hash);
return hash.to_string();
}

fn chiffrement (entree: String, sal: String) -> (String , String) {
let key = Aes256Gcm::generate_key(&mut OsRng);

let mut keystring= "".to_owned();
let vir = " ";
for a in key{
    let cle = a.to_string();
    keystring = keystring + &cle + vir ;
}

let cipher = Aes256Gcm::new(&key);
let mut valeurs   = Vec::new();
let mut crypt ="".to_owned();

//Mettre un sel dans la variable
let salt = sal.as_bytes();
let nonce = Nonce::from_slice(salt); // 96-bits; unique per message

//Mettre l'entréé à chiffrer dans la variable 
let preplain = entree;
let plaintext = preplain.as_bytes();
let ciphertext =cipher.encrypt(nonce, plaintext.as_ref());

match ciphertext{
    Ok(n) => valeurs=n,
    Err(..) => {}
}

for numbers in valeurs{
    let texte = numbers.to_string();
    crypt=crypt+&texte;
}
//La sortie est crypt, il s'agit d'un string correspondant à une suite de chiffre 
println!("{}", crypt);
return (crypt,keystring);
}


async fn bdd_create(form_data: web::Form<FormData>) -> std::result::Result<HttpResponse, Box<dyn std::error::Error>>{
    println!("salut les copains");
    let url = "mysql://lolo:lolo@localhost:3306/loic";
    let pool = Pool::new(url)?;
    let mut conn = pool.get_conn()?;
    let hashlolo = form_data.thing_to_show.to_string(); 
    println!("ceci est le hash lolo create: {}", hashlolo);   
    let log = form_data.thing_to_show2.to_string();    
    let sellolo= form_data.thing_to_show3.to_string();
    let y= salt();
    let sel= y.clone();
    let concat = hashlolo + &y.to_string();
    let x = passwordhash(y, concat);
    let presalt = Alphanumeric.sample_string(&mut rand::thread_rng(), 12);
    let salt_gcm = presalt.clone();
    let (aes , key) = chiffrement(x , presalt);
    println!("{} chiffrement aes:", aes);
    println!("{} la clefffffffff", key);
  	
    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS password (
            sel_1 text not null,
            sel_2 text not null,
            sel_gcm text not null,
            clefs text not null,
            login text not null,
            password text not null
        )")?;
    let _passwords = vec![
        Password { sel_1: sellolo  , sel_2:sel , sel_gcm:salt_gcm , clefs:key,  login: log , passw: aes },
    ];


    conn.exec_batch(
        r"INSERT INTO password (sel_1, sel_2, sel_gcm, clefs, login, password)
          VALUES (:sel_1, :sel_2, :sel_gcm, :clefs, :login, :password)",
        _passwords.iter().map(|p| params! {
            "sel_1" => &p.sel_1,
            "sel_2" => &p.sel_2,
            "sel_gcm" => &p.sel_gcm,
            "clefs" => &p.clefs,
            "login" => &p.login,
            "password" => &p.passw,
        })
    )?;
Ok(HttpResponse::Ok().content_type("text/html").body("super"))
 
 }
 
//////////////////////////////////////////

#[derive(Debug)] // Macro qui implémente l'erreur
pub struct MyError(String); // <-- needs debug and display
 impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A validation error occured on the input.")
    }
} 

impl ResponseError for MyError {} // <-- key // je crée l'instance erreur avec la macro du dessus

#[derive(Template)] //déclaration de la 1ére template html (le formulaire)
#[template(path = "authentification.html")]
struct Index {}

#[derive(Template)] // déclaration de la 2éme template html (affichage de la variable)
#[template(path = "show.html")]
struct Show {
    thing_to_show: String,
    thing_to_show2: String,
    thing_to_show3: String
}

#[derive(Deserialize)] // pour adapter la donnée
struct FormData {
    thing_to_show:String,
    thing_to_show2:String,
    thing_to_show3: String
}


async fn default_handler(req_method: Method) -> Result<impl Responder> {
    match req_method {
        Method::GET => {
            let file = NamedFile::open("templates/404.html")?
                .customize()
                .with_status(StatusCode::NOT_FOUND);
            Ok(Either::Left(file))
        }
        _ => Ok(Either::Right(HttpResponse::MethodNotAllowed().finish())),
    }
}

/* FONCTION EN PLUS DE THOMAS */
async fn showthis(form_data: web::Form<FormData>) -> Result<NamedFile> { //fonction pour afficher le 2éme rendu html
    let html = Show{ thing_to_show: form_data.thing_to_show.to_string(),thing_to_show2: form_data.thing_to_show2.to_string(),thing_to_show3: form_data.thing_to_show3.to_string()}.render().unwrap();
    println!("{}",html);
    let response = bdd_create(form_data).await?; // appel de la fonction bdd_create
    println!("{:?}", response.body());
    let path: PathBuf = "templates/menushowthis.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}


#[get("templates/menu1")]
async fn menu1(req: HttpRequest) -> Result<HttpResponse> {
    println!("{req:?}");
    // response
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::plaintext())
        .body(include_str!("../templates/menu1.html")))
}

const INDEX_HTML: &str = r#"
<html>
    <head>
        <title>Username B!frost</title>
        <script src="/templates/js/lottie-player.js" type="text/javascript"></script>
        <link rel="stylesheet" type="text/css" href="/templates/css/connexion_user.css" />
        <link rel="stylesheet" type="text/css" href="css/first_login.css">
        <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/4.7.0/css/font-awesome.min.css">
        <link href='https://fonts.googleapis.com/css?family=Titillium+Web:400,300,600' rel='stylesheet' type='text/css'>  
        <link href='https://fonts.googleapis.com/css?family=Titillium+Web:400,300,600' rel='stylesheet' type='text/css'> 
    </head>
    <style>
    lottie-player {
        width: 370px;
        height: 370px;
      }
    </style>
    <body class="body">     
    <div class="login-page">
      <div class="form">
      <div class="titre">
      <h1 align="center">SIGNUP BIFROST</h1>
        </div>
        <form id="my-form" action="/showthis2" method="post">
      <lottie-player
      src="https://assets4.lottiefiles.com/datafiles/XRVoUu3IX4sGWtiC3MPpFnJvZNq7lVWDCa8LSqgS/profile.json"
      background="transparent"
      speed="1"
      style="justify-content: center"
      loop
      autoplay
    ></lottie-player>
          <input name="thing" id="username" placeholder="&#xf007;  username" />
          <input type="submit" value="Login" style="background-color: #B22222; color: white;">
      </form>
      </div>
    </div>
  </body>
</html>
"#;

const SHOW_HTML: &str = r#"
<html>
    <head>
        <title>Username B!frost</title>
        <script>
            // Rediriger automatiquement vers la page suivante
            window.location.replace('http://127.0.0.1:8080/connexion_mdp');
        </script>
    </head>
    <body>
        <h1>Showing thing:</h1>
    </body>
</html>
"#;


#[derive(Deserialize)]
struct FormData2 {
    thing: String,
}

async fn index2() -> Result<HttpResponse, MyError> {
    Ok(HttpResponse::Ok().content_type("text/html").body(INDEX_HTML))
}

fn dechiffrement (ki: String , entree: String, sal: String) -> String {
let result: Vec<&str> = ki.split(" ").collect();
let mut array = GenericArray::generate(|i: usize| i as u8);
    let mut i = 0;
    for a in result{
    	if i<32{
    		let my_string = a.to_string();
    		let my_int= my_string.parse::<u8>().unwrap();
    		array[i] = my_int;
    		i = i+1;
    	}
    }
    
let cipher = Aes256Gcm::new(&array);
let mut valeurs   = Vec::new();
let mut crypt ="".to_owned();

//Mettre un sel dans la variable
let salt = sal.as_bytes();
let nonce = Nonce::from_slice(salt); // 96-bits; unique per message

//Mettre l'entréé à chiffrer dans la variable 
let preplain = entree;
let plaintext = preplain.as_bytes();
let ciphertext =cipher.encrypt(nonce, plaintext.as_ref());

match ciphertext{
    Ok(n) => valeurs=n,
    Err(..) => {}
}

for numbers in valeurs{
    let texte = numbers.to_string();
    crypt=crypt+&texte;
}
//La sortie est crypt, il s'agit d'un string correspondant à une suite de chiffre 
println!("{}", crypt);
return crypt;
}


#[derive(Debug, Deserialize)]
struct MyForm {
    hash: String,
    //password: String
}

async fn index3() -> impl Responder {
    let value = unsafe { THING_TO_SHOW.clone() }; // call the unsafe function to obtain the value of SEL_HTML
    let valuebis = value.unwrap_or_default();
    println!("Nom: {}", valuebis);
    println!("Hello world");
    let url = "mysql://lolo:lolo@localhost:3306/loic";
    let pool = match Pool::new(url) {
        Ok(pool) => pool,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Error connecting to the database: {}", e));
        }
    };
    let mut conn = match pool.get_conn() {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Error getting a connection from the pool: {}", e));
        }
    };
    let query = format!("SELECT * FROM password WHERE login = '{}'", valuebis);

    let selected_passwords = match conn.query_map(
        &query,
        |(sel_1, sel_2, sel_gcm, clefs, login, passw)| {
            Password {
                sel_1,
                sel_2,
                sel_gcm,
                clefs,
                login,
                passw,
            }
        },
    ) {
        Ok(results) => results,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Error executing the query: {}", e));
        }
    };

    let mut log_extrait = "".to_string();

    for burnout in selected_passwords {
        log_extrait.push_str(&burnout.sel_1);
    }

    unsafe {
        SEL_HTML = Some(log_extrait.clone());
        println!("I'm the SEL :{:?}", SEL_HTML);
    }

    let html_content = html();
    HttpResponse::Ok().body(html_content)
}


static mut THING_TO_SHOW: Option<String> = None;


async fn showthis2(form_data: web::Form<FormData2>) -> Result<HttpResponse, MyError> {
    let html = SHOW_HTML
        .replace("{{thing_to_show}}", &form_data.thing);
        println!("Envoie le nom d'utilisateur");
        println!("Le nom d'utilisateur est :{}",form_data.thing);
        //let path: PathBuf = "http://127.0.0.1:8080/index%22.parse().unwrap();
        unsafe {
            THING_TO_SHOW = Some(form_data.thing.clone()); // stockage de form_data.thing dans la variable globale THING_TO_SHOW
        }
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

static mut SEL_HTML: Option<String> = None;

async fn on_submit_form(form: web::Form<MyForm>) -> /*impl Responder*/std::result::Result<NamedFile, Box<dyn std::error::Error>> {
    let value = unsafe { THING_TO_SHOW.clone() }; // call the unsafe function to obtain the value of SEL_HTML
    let valuebis = value.unwrap_or_default();
      let url = "mysql://lolo:lolo@localhost:3306/loic";
      let pool = Pool::new(url)?;
      let mut conn = pool.get_conn()?;
      let query = format!("SELECT * FROM password WHERE login = '{}'", valuebis);	
      println!("Test");
  
      let hash = &form.hash;
          let selected_passwords2 = conn
          .query_map(
              &query,
              |(sel_1, sel_2, sel_gcm, clefs, login, passw)| {
                  Password { sel_1, sel_2, sel_gcm, clefs, login, passw }
              },
          )?;
              let selected_passwords3 = conn
          .query_map(
              &query,
              |(sel_1, sel_2, sel_gcm, clefs, login, passw)| {
                  Password { sel_1, sel_2, sel_gcm, clefs, login, passw }
              },
          )?;
              let selected_passwords4 = conn
          .query_map(
              &query,
              |(sel_1, sel_2, sel_gcm, clefs, login, passw)| {
                  Password { sel_1, sel_2, sel_gcm, clefs, login, passw }
              },
          )?;
              let selected_passwords5 = conn
          .query_map(
              &query,
              |(sel_1, sel_2, sel_gcm, clefs, login, passw)| {
                  Password { sel_1, sel_2, sel_gcm, clefs, login, passw }
              },
          )?;
          
          
  let mut log_extrait_sel_backend = "".to_string();
  let mut log_extrait_password = "".to_string();
  let mut log_extrait_sel_gcm = "".to_string();
  let mut log_extrait_clefs_aes = "".to_string();
  
  
  
  for salting in selected_passwords2 {
  log_extrait_sel_backend.push_str(&salting.sel_2);
  }
  
  for motdepasse in selected_passwords3 {
  log_extrait_password.push_str(&motdepasse.passw);
  }
  
  for gcmsuite in selected_passwords4 {
  log_extrait_sel_gcm.push_str(&gcmsuite.sel_gcm);
  }
  
  for aesfinit in selected_passwords5 {
  log_extrait_clefs_aes.push_str(&aesfinit.clefs);
  }

  let y= log_extrait_sel_backend;
  let concat = hash.to_owned() + &y.to_string();
  let x = passwordhash(y ,concat);
  let aes = dechiffrement(log_extrait_clefs_aes, x , log_extrait_sel_gcm); 
        
    if log_extrait_password != aes{
      
          println!("oh tes fatiguer minot"); //insere la fonction retour la 
          let path_e: PathBuf ="./templates/erreur_mdp.html".into();
          println!("voici le path {:?}", path_e);
          Ok(NamedFile::open(path_e)?)
      }
      else {
      //HttpResponse::Found().header("LOCATION", "/templates/menu1.html").finish()
      let path: PathBuf="./templates/connexion_reussi.html".into();
      Ok(NamedFile::open(path)?)}
  }

//page3
fn html() -> String {
    let sellallegreti = unsafe { SEL_HTML.clone() }; // call the unsafe function to obtain the value of SEL_HTML
    let sellallegreti_str = sellallegreti.unwrap_or_default();
    let html = format!(r#"
        <html>
        <head>
        <meta charset="UTF-8">
        <title>Conexion B!frost</title>
        <script src="/templates/js/lottie-player.js" type="text/javascript"></script>
        <link rel="stylesheet" type="text/css" href="/templates/css/connexion_mdp.css" />
        <link rel="stylesheet" type="text/css" href="css/first_login.css">
        <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/4.7.0/css/font-awesome.min.css">
        <link href='https://fonts.googleapis.com/css?family=Titillium+Web:400,300,600' rel='stylesheet' type='text/css'>  
        <link href='https://fonts.googleapis.com/css?family=Titillium+Web:400,300,600' rel='stylesheet' type='text/css'> 
       </head>
        <script src="/templates/js/sha256.js" type="text/javascript"></script>
        <script>
            function t(){{
                if (document.getElementsByName("hash_html")[0].value == "" ) {{
                    alert("Veuillez entrer un mot de passe.");
                    return false;
                }}
                else {{
                    var mdp = document.getElementsByName("hash_html")[0].value ;
                    var mdphash = document.getElementsByName("hash_html")[0].value + "{sellallegreti_str}".toString();
                    let hash = sha256(mdphash)
                    let hash_input = document.getElementsByName("hash")[0];
                    hash_input.setAttribute("value", hash);
                    return true;
                }}
            }}

       

        </script>
        <style>
        lottie-player {{
            width: 370px;
            height: 370px;
          }}
        </style>
            <body class="body">
                <div class="login-page">
                    <div class="form">
                        <div class="titre">
                            <h1 align="center">CONEXION BIFROST</h1>
                        </div>


                <form action="/submit" method="post" onsubmit="return t();">
                <lottie-player
                src="https://assets4.lottiefiles.com/datafiles/XRVoUu3IX4sGWtiC3MPpFnJvZNq7lVWDCa8LSqgS/profile.json"
                background="transparent"
                speed="1"
                style="justify-content: center"
                loop
                autoplay
              ></lottie-player>
                    <input type="password" name="hash_html" placeholder="&#xf023;  mdp">
                    <input type="hidden" name="hash" >
                    <input type="submit" value="Login" style="background-color: #B22222; color: white;">
                </form>
                </div>
    </div>
            </body>
        </html>
    "#, sellallegreti_str=sellallegreti_str);
    println!("I'm the SELLE :{:?}", sellallegreti_str);

    html
}

async fn submit_form(form: web::Form<MyForm>) -> impl Responder {
    let hash = &form.hash;
    //let password = &form.password;
    println!("Le hash html est: {}", hash);
    let _salt_value = "salut".to_string(); 
    //println!("{}", password);
    HttpResponse::Found().header("LOCATION", "/templates/menu1.html").finish()
}
/////////////////////////////////////////BDD//////////////////////////////////////////////////
#[derive(Deserialize)]
struct BDD {
    login: String,
}
//////////////////////////////////////////PAGE BDD///////////////////////////////////////////////////////////

fn see_bdd(result: Vec<String>) -> String {
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <title>BDD USER Bifrost</title>
            <style>
                body {{
                    background-image: url('templates/css/BDD4.jpg');
                    background-repeat: no-repeat;
                    background-size: cover;
                    text-align: center;
                }}
                h1 {{
                    text-align: center;
                    margin-top: 50px;
                    margin-bottom: 20px;
                    font-size: 50px;
                    color: white;
                }}
                table {{
                    border-collapse: collapse;
                    width: 50%;
                    margin: 0 auto;
                    border: 1px solid white;
                }}
                th, td {{
                    text-align: center;
                    padding: 8px;
                    font-size: 40px;
                    color: white;
                    border: 1px solid white;
                    
                }}
                th {{
                    text-align: center;
                    color: white;
                    font-size: 50px;
                }}
                input[type="button"] {{
                    margin: 10px;
                    background-color: white;
                    color: black;
                    border: none;
                    padding: 10px 20px;
                    font-size: 20px;
                    cursor: pointer;
                    border-radius: 5px;
                }}
                #retour {{
                    position: absolute;
                    top: 20px;
                    left: 20px;
                    background-color: white;
                    color: black;
                    border: none;
                    padding: 10px 20px;
                    font-size: 20px;
                    cursor: pointer;
                    border-radius: 5px;
                }}
            </style>
        </head>
        <body>
            <h1>USER B!FROST</h1>
            <input type="button" id="retour" onclick="window.location.href='http://127.0.0.1:8080/templates/menu1.html'" value="Back">
            <input type="button" onclick="window.location.href='http://127.0.0.1:8080/delete'" value="Delete User">
            <input type="button" onclick="window.location.href='http://127.0.0.1:8080/templates/ajouter_user.html'" value="Add User">
            <table>
                <tr>
                    <th>Login</th>
                </tr>
                {}
            </table>
            <script>
                // Bloquer l'accès à la flèche retour
                window.history.pushState(null, null, '');
                window.addEventListener('popstate', function (event) {{
                    window.history.pushState(null, null, '');
                }});
                
                // Bloquer l'accès à une nouvelle URL
                const url = "http://127.0.0.1:8080/ADD";
                if (window.location.href === url) {{
                    console.log("test");
                    window.location.replace("http://127.0.0.1:8080/BDD"); // rediriger l'utilisateur vers une page d'erreur
                }}
            </script>
        </body>
        </html>
        "#,
        result
        .into_iter()
        .map(|login| { // utilise la première valeur du tuple (le login)
            format!(
                r#"
                    <tr>
                        <td>{}</td>
                    </tr>
                "#,
                login
            )
        })
        .collect::<String>()
    );

    html
}



async fn BDD(db: web::Data<Pool>) -> impl Responder {
    let mut conn = db.get_conn().unwrap();

    let query = "SELECT login FROM password";
    let result = conn.query_map(query, |login: String| login).unwrap();
    let html_content = see_bdd(result);
    HttpResponse::Ok().body(html_content)
}

///////////////////////////////////////////SUPPRIMER ///////////////////////////////////////////////////////////

fn supprimer() -> &'static str {
    "<!DOCTYPE html>
    <html>
        <head>
            <meta charset=\"UTF-8\">
            <title>Delete user Bifrost</title>
            <style>
                body {
                    margin-top: 20%;
                    background-image: url('templates/css/img.png');
                    background-repeat: no-repeat;
                    background-size: cover;
                }
                h1 {
                    text-align: center;
                    color: #A52A2A;
                    text-decoration: underline solid #A52A2A;
                    font-family: Impact, 'Arial Black', Arial, Verdana, sans-serif;
                    letter-spacing: 5px;
                    font-size: 40px;
                }
                .input {
                    font-family: FontAwesome, 'Roboto', sans-serif;
                    outline: 0;
                    background: #f2f2f2;
                    width: 22%;
                    border: 0;
                    margin-left: 40%;
                    padding: 15px;
                    box-sizing: border-box;
                    font-size: 14px;
                  border-radius:10px;
                  display: block;
                    
                  }
                .button{
                    font-family: 'Titillium Web', sans-serif;
                    font-size: 14px;
                    font-weight: bold;
                    letter-spacing: .1em;
                    outline: 0;
                    background: #B22222;
                    width: 22%;
                    border: 0;
                    border-radius:30px;
                    margin-left: 40%;
                    padding: 15px;
                    color: #FFFFFF;
                    -webkit-transition: all 0.3 ease;
                    transition: all 0.3 ease;
                    cursor: pointer;
                    transition: all 0.2s;
                    display: block;
                  }

                .label{
                    text-align: center;
                    color: #A52A2A;
                    font-size: 20px;
                    font-weight: bold;
                    font-family: Impact, 'Arial Black', Arial, Verdana,  
                    display: block;
                    margin-left: 40%;
                }
            </style>
        </head>
        <body>
            <h1>User to delete</h1>
            <form id=\"my-form\">
                <br>
                <label class=\"label\" for=\"login\">User to delete :</label>
                <br>
                <br>
                <input type=\"text\" id=\"login\" class=\"input\" name=\"login\">
                <br>
                <button class=\"button\" type=\"submit\">Delete</button>
            </form>
            <script>
                const form = document.getElementById(\'my-form\');
                form.addEventListener(\'submit\', (event) => {
                    event.preventDefault();
                    const formData = new FormData(form);
                    fetch(\'/delete\', {
                        method: \'POST\',
                        headers: {
                            \'Content-Type\': \'application/x-www-form-urlencoded\'
                        },
                        body: new URLSearchParams(formData)
                    })
                    .then(response => {
                        // traitement de la réponse ici
                        console.log(response.text());
                        // redirection de l'utilisateur
                        window.location.href = \'http://127.0.0.1:8080/templates/menu1.html\';
                    })
                    .catch(error => console.error(error));
                });
            </script>
        </body>
    </html>"
}


async fn delete_user() -> io::Result<impl Responder> {
    let html = supprimer();
    Ok(HttpResponse::Ok().body(html))
}


async fn delete(db: web::Data<Pool>, form_data: web::Form<BDD>) -> HttpResponse {
    let login = form_data.login.to_string();
    println!("Login: {}", login);
        
    let mut conn = db.get_conn().unwrap();
    conn.exec_drop(
        r"DELETE FROM password WHERE login = :login",
        params! {
            "login" => login,
            },
    ).unwrap();

    HttpResponse::Found()
        .header("Location", "/")
        .finish()
}
///////////////////////////////////////////BDD credential///////////////////////////////////////////////////////////////

#[derive(Deserialize)]
struct Credential {
    name: String,
    value : String,
}

//////////////////////////////////////////PAGE BDD///////////////////////////////////////////////////////////

fn see_bdd2(result: Vec<(String,String)>) -> String {
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <style>
                body {{
                    background-image: url('templates/css/BDD4.jpg');
                    background-repeat: no-repeat;
                    background-size: cover;
                    text-align: center;
                }}
                h1 {{
                    text-align: center;
                    margin-top: 50px;
                    margin-bottom: 20px;
                    font-size: 50px;
                    color: white;
                }}
                table {{
                    border-collapse: collapse;
                    width: 50%;
                    margin: 0 auto;
                    border: 1px solid white;
                }}
                th, td {{
                    text-align: center;
                    padding: 8px;
                    font-size: 40px;
                    color: white;
                    border: 1px solid white;
                    
                }}
                th {{
                    text-align: center;
                    color: white;
                    font-size: 50px;
                }}
                input[type="button"] {{
                    margin: 10px;
                    background-color: white;
                    color: black;
                    border: none;
                    padding: 10px 20px;
                    font-size: 20px;
                    cursor: pointer;
                    border-radius: 5px;
                }}
                #retour {{
                    position: absolute;
                    top: 20px;
                    left: 20px;
                    background-color: white;
                    color: black;
                    border: none;
                    padding: 10px 20px;
                    font-size: 20px;
                    cursor: pointer;
                    border-radius: 5px;
                }}
            </style>
    
            <head>
                <meta charset="UTF-8">
                <title>BDD USER Bifrost</title>
                <link rel="stylesheet" type="text/css" href="templates/marre.css">
                <style>
                    table {{
                        border-collapse: collapse;
                    }}
                    table, th, td {{
                        border: 1px solid black;
                    }}
                    caption {{
                        text-align: center;
                    }}
                </style>
            </head>
            <body>
                <input type="button" id="retour" onclick="window.location.href='http://127.0.0.1:8080/templates/menu1.html'" value="Back">
                <h1>Database credentials</h1>
                <input type="button" onclick="window.location.href='http://127.0.0.1:8080/ADD'" value="Add credential">
                <input type="button" onclick="window.location.href='http://127.0.0.1:8080/delete_credential'" value="Delete credential">
                <input type="button" onclick="window.location.href='http://127.0.0.1:8080/modifier_credential'" value="Modification credential">
                <table>
                    <tr>
                        <th>Nom</th>
                        <th>Value</th>
                    </tr>
                    {}
                </table>
                <script>
                    // Bloquer l'accès à la flèche retour
                    window.history.pushState(null, null, '');
                    window.addEventListener('popstate', function (event) {{
                        window.history.pushState(null, null, '');
                    }});
                    
                    // Bloquer l'accès à une nouvelle URL
                    const url = "http://127.0.0.1:8080/ADD";
                    if (window.location.href === url) {{
                    console.log("test")
                    window.location.replace("http://127.0.0.1:8080/BDD"); // rediriger l'utilisateur vers une page d'erreur
                    }}
                </script>
            </body>
        </html>
        "#,
        result
        .into_iter()
        .map(|(nom,value)| { // utilise la première valeur du tuple (le login)
            format!(
                r#"
                    <tr>
                        <td>{}</td>
                        <td>{}</td>
                    </tr>
                "#,
                nom,value
            )
        })
        .collect::<String>()
    );

    html
}



async fn bdd(db: web::Data<Pool>) -> impl Responder {
    let mut conn = db.get_conn().unwrap();

    let query = "SELECT nom , value FROM credential";
    let result = conn.query_map(query, |(nom, value)| (nom, value)).unwrap();
    let html_content = see_bdd2(result);
    HttpResponse::Ok().body(html_content)
}

//////////////////////////////////////////ADD/////////////////////////////////////////////////////////////

fn ajouter_credential() -> &'static str {
    "<!DOCTYPE html>
    <html>
        <head>
            <meta charset=\"UTF-8\">
            <title>BDD credential Bifrost</title>
        </head>
        <style>
                body {
                    margin-top: 20%;
                    background-image: url('templates/css/img.png');
                    background-repeat: no-repeat;
                    background-size: cover;
                }
                h1 {
                    text-align: center;
                    color: #A52A2A;
                    text-decoration: underline solid #A52A2A;
                    font-family: Impact, 'Arial Black', Arial, Verdana, sans-serif;
                    letter-spacing: 5px;
                    font-size: 40px;
                }
                #my-form input {
                    margin: 10px;
                    background-color: white;
                    color: black;
                    border: none;
                    padding: 10px 20px;
                    font-size: 16px;
                    cursor: pointer;
                    border-radius: 5px;
                }
                #my-form button {
                    margin: 10px;
                    background-color: white;
                    color: black;
                    border: none;
                    padding: 10px 20px;
                    font-size: 20px;
                    cursor: pointer;
                    border-radius: 5px;
                }
                #my-form label {
                    color: #FFFFFF;
                    display: block;
                    margin: 0 auto;
                    font-size: 24px;
                }
                #my-form {
                    text-align: center;
                }
            </style>
        <body>
        <h1>Add credentials</h1>
        <form id=\"my-form\">
            <label for=\"name\">Name:</label>
            <input type=\"text\" id=\"name\" name=\"name\">
            </br>
            </br>
            <label for=\"value\">Value :</label>
            <input type=\"text\" id=\"value\" name=\"value\">
            <br>
            </br>
            <button type=\"submit\">Add user</button>
        </form>
            <script>
                const form = document.getElementById('my-form');
                form.addEventListener('submit', (event) => {
                    event.preventDefault();
                    const formData = new FormData(form);
                    fetch('/ADD', {
                        method: 'POST',
                        headers: {
                'Content-Type': 'application/x-www-form-urlencoded'
                                 },
                        body: new URLSearchParams(formData)
    
                    })
                    .then(response => {
                        // traitement de la réponse ici
                        console.log(response.text());
                        // redirection de l'utilisateur
                        window.location.href = 'http://127.0.0.1:8080/BDD2';
                    })
                    .catch(error => console.error(error));
                });
        
            </script>
        </body>
    </html>"
}

async fn add_credential() -> io::Result<impl Responder> {
    let html = ajouter_credential();
    Ok(HttpResponse::Ok().body(html))
}


async fn ajout_cred(db: web::Data<Pool>, form_data: web::Form<Credential>) -> HttpResponse {
    let nom = form_data.name.to_string();
    println!("Nom: {}", nom);
    let value = form_data.value.to_string();
    println!("Value: {}", value);
    
    let mut conn = db.get_conn().unwrap();
    conn.exec_drop(
        r"INSERT INTO credential (nom, value) VALUES (:nom, :value)",
        params! {
            "nom" => nom,
            "value" => value,
        },
    ).unwrap();

    HttpResponse::Found()
        .header("Location", "/")
        .finish()
}

///////////////////////////////////////////SUPPRIMER ///////////////////////////////////////////////////////////

fn supprimer_credential() -> &'static str {
    "<!DOCTYPE html>
    <html>
        <head>
            <meta charset=\"UTF-8\">
            <title>Delete credentials</title>
        </head>
        <style>
                body {
                    margin-top: 20%;
                    background-image: url('templates/css/img.png');
                    background-repeat: no-repeat;
                    background-size: cover;
                }
                h1 {
                    text-align: center;
                    color: #A52A2A;
                    text-decoration: underline solid #A52A2A;
                    font-family: Impact, 'Arial Black', Arial, Verdana, sans-serif;
                    letter-spacing: 5px;
                    font-size: 40px;
                }
                #my-form input {
                    margin: 10px;
                    background-color: white;
                    color: black;
                    border: none;
                    padding: 10px 20px;
                    font-size: 16px;
                    cursor: pointer;
                    border-radius: 5px;
                }
                #my-form button {
                    margin: 10px;
                    background-color: white;
                    color: black;
                    border: none;
                    padding: 10px 20px;
                    font-size: 20px;
                    cursor: pointer;
                    border-radius: 5px;
                }
                #my-form label {
                    color: #FFFFFF;
                    display: block;
                    margin: 0 auto;
                    font-size: 24px;
                }
                #my-form {
                    text-align: center;
                }
            </style>
        <body>
        <h1>Delete credentials</h1>
        <form id=\"my-form\">
            <label for=\"name\">Name :</label>
            <input type=\"text\" id=\"name\" name=\"name\">
            <label for=\"value\">Value:</label>
            <input type=\"text\" id=\"value\" name=\"value\">
            <br>
            <button type=\"submit\">Delete</button>
        </form>
            <script>
                const form = document.getElementById('my-form');
                form.addEventListener('submit', (event) => {
                    event.preventDefault();
                    const formData = new FormData(form);
                    fetch('/delete_credential', {
                        method: 'POST',
                        headers: {
                'Content-Type': 'application/x-www-form-urlencoded'
                                 },
                        body: new URLSearchParams(formData)
    
                    })
                .then(response => {
                    // traitement de la réponse ici
                    console.log(response.text());
                    // redirection de l'utilisateur
                    window.location.href = 'http://127.0.0.1:8080/BDD2';
                })
                .catch(error => console.error(error));
            });
        
            </script>
        </body>
    </html>"
}

async fn delete_credential() -> io::Result<impl Responder> {
    let html = supprimer_credential();
    Ok(HttpResponse::Ok().body(html))
}


async fn delete_cred(db: web::Data<Pool>, form_data: web::Form<Credential>) -> HttpResponse {
    let nom = form_data.name.to_string();
    println!("Nom: {}", nom);
    let value = form_data.value.to_string();
    println!("Value: {}", value);
    
    let mut conn = db.get_conn().unwrap();
    conn.exec_drop(
        r"DELETE FROM credential WHERE nom = :nom AND value = :value",
        params! {
            "nom" => nom,
            "value" => value,
        },
    ).unwrap();

    HttpResponse::Found()
        .header("Location", "/")
        .finish()
}

///////////////////////////////////////////MODIFIER///////////////////////////////////////////////////////////

fn modify_credential() -> &'static str {
    "<!DOCTYPE html>
    <html>
        <head>
            <meta charset=\"UTF-8\">
            <title>Credentials modification</title>
        </head>
        <style>
                body {
                    margin-top: 20%;
                    background-image: url('templates/css/img.png');
                    background-repeat: no-repeat;
                    background-size: cover;
                }
                h1 {
                    text-align: center;
                    color: #A52A2A;
                    text-decoration: underline solid #A52A2A;
                    font-family: Impact, 'Arial Black', Arial, Verdana, sans-serif;
                    letter-spacing: 5px;
                    font-size: 40px;
                }
                #my-form input {
                    margin: 10px;
                    background-color: white;
                    color: black;
                    border: none;
                    padding: 10px 20px;
                    font-size: 16px;
                    cursor: pointer;
                    border-radius: 5px;
                }
                #my-form button {
                    margin: 10px;
                    background-color: white;
                    color: black;
                    border: none;
                    padding: 10px 20px;
                    font-size: 20px;
                    cursor: pointer;
                    border-radius: 5px;
                }
                #my-form label {
                    color: #FFFFFF;
                    display: block;
                    margin: 0 auto;
                    font-size: 24px;
                }
                #my-form {
                    text-align: center;
                }
        </style>
        <body>
        <h1>Crendentials to modify </h1>
        <form id=\"my-form\">
            <label for=\"name\">Name :</label>
            <input type=\"text\" id=\"name\" name=\"name\">
            <label for=\"value\">Value:</label>
            <input type=\"text\" id=\"value\" name=\"value\">
            <br>
            <button type=\"submit\">Modify</button>


        </form>
            <script>
                const form = document.getElementById('my-form');
                form.addEventListener('submit', (event) => {
                    event.preventDefault();
                    const formData = new FormData(form);
                    fetch('/modifier_credential', {
                        method: 'POST',
                        headers: {
                'Content-Type': 'application/x-www-form-urlencoded'
                                 },
                        body: new URLSearchParams(formData)
    
                    })

                    .then(response => {
                        // traitement de la réponse ici
                        console.log(response.text());
                        // redirection de l'utilisateur
                        window.location.href = 'http://127.0.0.1:8080/ADD2';
                    })
                    .catch(error => console.error(error));
                });
        
            </script>
        </body>
    </html>"
}


async fn modifier_credential() -> io::Result<impl Responder> {
    let html = modify_credential();
    Ok(HttpResponse::Ok().body(html))
}


async fn modifier_cred(db: web::Data<Pool>, form_data: web::Form<Credential>) -> HttpResponse {
    let nom = form_data.name.to_string();
    println!("Nom: {}", nom);
    let value = form_data.value.to_string();
    println!("Value: {}", value);
    
    let mut conn = db.get_conn().unwrap();
    conn.exec_drop(
        r"DELETE FROM credential WHERE nom = :nom AND value = :value",
        params! {
            "nom" => nom,
            "value" => value,
        },
    ).unwrap();

    HttpResponse::Found()
        .header("Location", "/")
        .finish()
}

fn modify_credential2() -> &'static str {
    "<!DOCTYPE html>
    <html>
        <head>
            <meta charset=\"UTF-8\">
            <title>Modification credentials</title>
        </head>
        <style>
                body {
                    margin-top: 20%;
                    background-image: url('templates/css/img.png');
                    background-repeat: no-repeat;
                    background-size: cover;
                }
                h1 {
                    text-align: center;
                    color: #A52A2A;
                    text-decoration: underline solid #A52A2A;
                    font-family: Impact, 'Arial Black', Arial, Verdana, sans-serif;
                    letter-spacing: 5px;
                    font-size: 40px;
                }
                #my-form input {
                    margin: 10px;
                    background-color: white;
                    color: black;
                    border: none;
                    padding: 10px 20px;
                    font-size: 16px;
                    cursor: pointer;
                    border-radius: 5px;
                }
                #my-form button {
                    margin: 10px;
                    background-color: white;
                    color: black;
                    border: none;
                    padding: 10px 20px;
                    font-size: 20px;
                    cursor: pointer;
                    border-radius: 5px;
                }
                #my-form label {
                    color: #FFFFFF;
                    display: block;
                    margin: 0 auto;
                    font-size: 24px;
                }
                #my-form {
                    text-align: center;
                }
        </style>
        <body>
        <h1>Enter new information</h1>
        <form id=\"my-form\">
            <label for=\"name\">Name :</label>
            <input type=\"text\" id=\"name\" name=\"name\">
            <label for=\"value\">Value:</label>
            <input type=\"text\" id=\"value\" name=\"value\">
            <br>

            <button type=\"submit\">Add credential</button>

        </form>
            <script>
                const form = document.getElementById('my-form');
                form.addEventListener('submit', (event) => {
                    event.preventDefault();
                    const formData = new FormData(form);
                    fetch('/ADD2', {
                        method: 'POST',
                        headers: {
                'Content-Type': 'application/x-www-form-urlencoded'
                                 },
                        body: new URLSearchParams(formData)
    
                    })
                    
                    .then(response => {
                        // traitement de la réponse ici
                        console.log(response.text());
                        // redirection de l'utilisateur
                        window.location.href = 'http://127.0.0.1:8080/BDD2';
                    })
                    .catch(error => console.error(error));
                });
            </script>
        </body>
    </html>"
}

async fn add_credential2() -> io::Result<impl Responder> {
    let html = modify_credential2();
    Ok(HttpResponse::Ok().body(html))
}


async fn ajout_credential2(db: web::Data<Pool>, form_data: web::Form<Credential>) -> HttpResponse {
    let nom = form_data.name.to_string();
    println!("Nom: {}", nom);
    let value = form_data.value.to_string();
    println!("Value: {}", value);
    
    let mut conn = db.get_conn().unwrap();
    conn.exec_drop(
        r"INSERT INTO credential (nom, value) VALUES (:nom, :value)",
        params! {
            "nom" => nom,
            "value" => value,
        },
    ).unwrap();

    HttpResponse::Found()
        .header("Location", "/")
        .finish()
}

///////////////////////////////////////////Changer password ///////////////////////////////////////////////////////////

fn changer() -> &'static str {
    "<!DOCTYPE html>
    <html>
        <head>
            <meta charset=\"UTF-8\">
            <title>Change password</title>
            <style>
            body {
                margin-top: 20%;
                background-image: url('templates/css/img.png');
                background-repeat: no-repeat;
                background-size: cover;
            }
            h1 {
                text-align: center;
                color: #A52A2A;
                text-decoration: underline solid #A52A2A;
                font-family: Impact, 'Arial Black', Arial, Verdana, sans-serif;
                letter-spacing: 5px;
                font-size: 40px;
            }
            .input {
                font-family: FontAwesome, 'Roboto', sans-serif;
                outline: 0;
                background: #f2f2f2;
                width: 22%;
                border: 0;
                margin-left: 40%;
                padding: 15px;
                box-sizing: border-box;
                font-size: 14px;
              border-radius:10px;
              display: block;
                
              }
            .button{
                font-family: 'Titillium Web', sans-serif;
                font-size: 14px;
                font-weight: bold;
                letter-spacing: .1em;
                outline: 0;
                background: #B22222;
                width: 22%;
                border: 0;
                border-radius:30px;
                margin-left: 40%;
                padding: 15px;
                color: #FFFFFF;
                -webkit-transition: all 0.3 ease;
                transition: all 0.3 ease;
                cursor: pointer;
                transition: all 0.2s;
                display: block;
              }

            .label{
                text-align: center;
                color: #A52A2A;
                font-size: 20px;
                font-weight: bold;
                font-family: Impact, 'Arial Black', Arial, Verdana,  
                display: block;
                margin-left: 40%;
            }
        </style>
        </head>
        <body>
        <h1>Change User Bifrost</h1>
        <form id=\"my-form\">
            <br>
            <label class=\"label\" for=\"login\">User to modify :</label>
            <br>
            <br>
            <input class=\"input\" type=\"text\" id=\"login\" name=\"login\">
            <br>
            <button class=\"button\" type=\"submit\">Modify</button>
        </form>
            <script>
                const form = document.getElementById('my-form');
                form.addEventListener('submit', (event) => {
                    event.preventDefault();
                    const formData = new FormData(form);
                    fetch('/delete', {
                        method: 'POST',
                        headers: {
                'Content-Type': 'application/x-www-form-urlencoded'
                                 },
                        body: new URLSearchParams(formData)
    
                    })
                .then(response => {
                    // traitement de la réponse ici
                    console.log(response.text());
                    // redirection de l'utilisateur
                    window.location.href = 'http://127.0.0.1:8080/templates/modifier_user.html';
                })
                .catch(error => console.error(error));
            });
            </script>
        </body>
    </html>"
}

async fn change_user() -> io::Result<impl Responder> {
    let html = changer();
    Ok(HttpResponse::Ok().body(html))
}


async fn change(db: web::Data<Pool>, form_data: web::Form<BDD>) -> HttpResponse {
    let login = form_data.login.to_string();
    println!("Login: {}", login);
        
    let mut conn = db.get_conn().unwrap();
    conn.exec_drop(
        r"DELETE FROM password WHERE login = :login",
        params! {
            "login" => login,
            },
    ).unwrap();

    HttpResponse::Found()
        .header("Location", "/")
        .finish()
}

//////////MAIL BIFROST/////////////////////////////////////
async fn assistance() -> io::Result<impl Responder> {
    let html = include_str!("../templates/assistance.html");
    Ok(HttpResponse::Ok().body(html))
}

#[derive(Deserialize)]
struct Mail {
    nom: String,
    prenom: String,
    email: String,
    message: String,
}

async fn bifrost_mail(form_data: web::Form<Mail>) -> /*impl Responder*/std::result::Result<HttpResponse, Box<dyn std::error::Error>> {
    let nom= form_data.nom.to_string();
    println!("Nom: {}", nom);

    let prenom= form_data.prenom.to_string();
    println!("Prénom: {}", prenom);

    let adresse= form_data.email.to_string();
    println!("Adresse mail: {}", adresse);

    let message= form_data.message.to_string();
    println!("Message pour le destinataire: {}", message);

    //Code pour envoyer le mail
    let email = EmailBuilder::new()
    .to("bifrost83000@gmail.com")
    .from(adresse)
    .subject("Bonjour assistance")
    .text(message)
    .build()
    .unwrap();

    let mut mailer = SmtpClient::new_simple("smtp.gmail.com")
        .unwrap()
        .credentials(Credentials::new("bifrost83000".into(), "ypdq uxzk ueim wovs".into()))
        .transport();

    let result = mailer.send(email.into());
    println!("{:?}", result);

    Ok(HttpResponse::Ok().content_type("text/html").body("super"))
}
////////////////////////////////////////////MAIN/////////////////////////////////////////////////
#[actix_web::main]
async fn main() -> io::Result<()> {
    let db_url = "mysql://lolo:lolo@localhost:3306/loic";
    let db_pool = Pool::new(db_url).unwrap();
    let db_data = web::Data::new(db_pool);
    HttpServer::new(move || {
        App::new()
        .service(menu1) 
         // static files
        .service(Files::new("/templates", "templates").show_files_listing())
            // redirect
        .service(
                web::resource("/templates/login.html").route(web::get().to(|_req: HttpRequest| async move {
                    //println!("{req:?}");
                    HttpResponse::Found()
                        .insert_header((header::LOCATION, "templates/login.html"))
                        .finish()
                })),
            )
        .default_service(web::to(default_handler))
        .route("/showthis", web::post().to(showthis))
        .route("/connexion_user", web::get().to(index2))
        .route("/showthis2", web::post().to(showthis2))
        .route("/submit", web::post().to(on_submit_form))
        .route("/connexion_mdp", web::get().to(index3))
        .route("/submit", web::post().to(submit_form))
        .app_data(db_data.clone())
        .route("/BDD", web::get().to(BDD))
        .service(web::resource("/delete").route(web::get().to(delete_user)).route(web::post().to(delete)))
        .service(web::resource("/change").route(web::get().to(change_user)).route(web::post().to(change)))
        .service(web::resource("/assistance.html").route(web::get().to(assistance)).route(web::post().to(bifrost_mail)))
        .service(web::resource("/ADD").route(web::get().to(add_credential)).route(web::post().to(ajout_cred)))
        .route("/BDD2", web::get().to(bdd))
        .service(web::resource("/delete_credential").route(web::get().to(delete_credential)).route(web::post().to(delete_cred)))
        .service(web::resource("/modifier_credential").route(web::get().to(modifier_credential)).route(web::post().to(modifier_cred)))
        .service(web::resource("/ADD2").route(web::get().to(add_credential2)).route(web::post().to(ajout_credential2)))
            
    })
    .bind(("127.0.0.1", 8080))?
    .workers(2)
    .run()
    .await
}

