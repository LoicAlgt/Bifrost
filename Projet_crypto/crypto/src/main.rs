//Dépendances pour la page web 
use actix_web::{web, App, HttpServer, HttpResponse};
use actix_web::{get, Result};
use std::io;
use actix_files::NamedFile;
use serde::{Deserialize};
use std::path::PathBuf;
use actix_web::Responder;
use actix_files::Files;
use std::path::Path;

//Dépendances pour le mail
use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;

//Dépendance pour le nombre aléatoire
use rand::thread_rng;
use rand::Rng;

//pour mysql
use mysql::*;
use mysql::prelude::*;
use mysql::params;

//Pour faire des commande bash
use std::process::Command;

//Variable statique
use lazy_static::lazy_static;
use std::sync::Mutex;

//Fichier OCSP
use std::fs::File;
use std::io::prelude::*;


#[get("/{filename:.*}")]
async fn files(path: web::Path<(String,)>) -> Result<NamedFile> {
    Ok(NamedFile::open(format!("templates/{}", path.0))?)
}

#[derive(Deserialize)]
struct Information {
    adresse : String,
}

//Déclaration variable globale
lazy_static! {
    static ref RANDOM_NUMBER: Mutex<i32> = Mutex::new(0);
}

async fn info_mail(form_data: web::Form<Information>) -> /*impl Responder*/std::result::Result<HttpResponse, Box<dyn std::error::Error>> {
    let adresse= form_data.adresse.to_string();
    println!("Adresse mail: {}", adresse);

    
    let number = thread_rng().gen_range(10000..=30000);
    println!("{}",number);

    *RANDOM_NUMBER.lock().unwrap() = number;

    //Code pour envoyer le mail
    let email = EmailBuilder::new()
    .to(adresse)
    .from("crypto.projet2023@gmail.com")
    .subject("Example subject")
    .text(number.to_string())
    .build()
    .unwrap();

    let mut mailer = SmtpClient::new_simple("smtp.gmail.com")
        .unwrap()
        .credentials(Credentials::new("crypto.projet2023".into(), "kwhu qdat yrsy zjqb".into()))
        .transport();
        
    let result = mailer.send(email.into());
    println!("{:?}", result);

    Ok(HttpResponse::Ok().content_type("text/html").body("super"))
}

#[derive(Deserialize)]
struct Verif {
    code: String,
}

async fn on_submit_form(form_data: web::Form<Verif>) -> std::result::Result<NamedFile, Box<dyn std::error::Error>> {
    let value = form_data.code.to_string();
    println!("Code: {}", value); 

    let val= *RANDOM_NUMBER.lock().unwrap();
    let val_str = val.to_string();
    if val_str == value {
        println!("La valeur globale est égale à la valeur du formulaire.");
        let pathE: PathBuf ="./templates/code_bon.html".into();
        println!("voici le path {:?}", pathE);
        Ok(NamedFile::open(pathE)?)
    } 
    else {
        println!("La valeur globale est différente de la valeur du formulaire.");
        let path: PathBuf ="http://127.0.0.1:8080/code_bon.html".into();
        println!("voici le path {:?}", path);
        Ok(NamedFile::open(path)?)
    }
} 


///// Voir la BDD /////////////
fn see_bdd(result: Vec<(String,String,String,String,String,String,String)>) -> String {
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <style>
        table {{
            border-collapse: collapse;
            width: 100%;
        }}
        th, td {{
            text-align: left;
            padding: 8px;
        }}
        th {{
            background-color: #4CAF50;
            color: white;
        }}
        tr:nth-child(even) {{
            background-color: #f2f2f2;
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
                <caption>Liste des noms et prénoms</caption>
                <input type="button" onclick="window.location.href='http://127.0.0.1:8080/file'" value="Fichier Serveur OCSP">
                <input type="button" onclick="window.location.href='http://127.0.0.1:8080/MenuCrypto.html'" value="Retour">
                <table>
                    <tr>
                        <th>Nom</th>
                        <th>Prenom</th>
                        <th>Code</th>
                        <th>Adresse</th>
                        <th>pays</th>
                        <th>ville</th>
                        <th>organisation</th>
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
        .map(|(nom,prenom,code,adresse,pays,ville,organisation)| { // utilise la première valeur du tuple (le login)
            format!(
                r#"
                <tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>
                "#,
                nom,prenom,code,adresse,pays,ville,organisation
            )
        })
        .collect::<String>()
    );

    html
}

#[get("/file")]
async fn get_file() -> HttpResponse {
    let mut file = File::open("Serveur/inter/index.txt").expect("Impossible d'ouvrir le fichier");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Impossible de lire le contenu du fichier");
    HttpResponse::Ok().body(contents)
}

async fn BDD(db: web::Data<Pool>) -> impl Responder {
    let mut conn = db.get_conn().unwrap();

    let query = "SELECT * FROM utilisateurs";
    let result = conn.query_map(query, |(nom,prenom,code,adresse,pays,ville,organisation)|(nom,prenom,code,adresse,pays,ville,organisation)).unwrap();
    let html_content = see_bdd(result);
    HttpResponse::Ok().body(html_content)
}

////Ajout dans la BDD/////////////////////
async fn certificat() -> io::Result<impl Responder> {
    let html = include_str!("../templates/CA.html");
    Ok(HttpResponse::Ok().body(html))
}

#[derive(Deserialize)]
struct Certificat {
    nom: String,
    prenom: String,
    pays: String,
    region: String,
    ville: String,
    organisation: String,
    service: String,
    adresse: String,
    mdp: String,
}

fn ma_fonction(db: web::Data<Pool>, form_data: web::Form<Certificat>) {
    // Le corps de la fonction ici
    let nom = form_data.nom.to_string();
    let prenom = form_data.prenom.to_string();
    let val= *RANDOM_NUMBER.lock().unwrap();
    let code = val.to_string();
    println!("{}",code);
    let pays = form_data.pays.to_string();
    let region = form_data.region.to_string();
    let ville = form_data.ville.to_string();
    let organisation = form_data.organisation.to_string();
    let service = form_data.service.to_string();
    let adresse = form_data.adresse.to_string();
    let mut conn = db.get_conn().unwrap();
    conn.exec_drop(
        r"INSERT INTO utilisateurs (nom,prenom,code,adresse,pays,ville,organisation) VALUES (:nom,:prenom, :code, :adresse, :pays, :ville, :organisation)",
        params! {
            "nom" => nom,
            "prenom" => prenom,
            "code" => code,
            "adresse" => adresse,
            "pays" => pays,
            "ville" => ville,
            "organisation" => organisation,
        },
    ).unwrap();
}


async fn create_certificat(db: web::Data<Pool>, form_data: web::Form<Certificat>) -> HttpResponse {
    let nom = form_data.nom.to_string();
    println!("Nom: {}", nom);
    let prenom = form_data.prenom.to_string();
    println!("Prénom: {}", prenom);
    let val= *RANDOM_NUMBER.lock().unwrap();
    let code = val.to_string();
    println!("Code : {}",code);
    let pays = form_data.pays.to_string();
    println!("Pays: {}", pays);
    let region = form_data.region.to_string();
    println!("Region: {}", region);
    let ville = form_data.ville.to_string();
    println!("Ville: {}", ville);
    let organisation = form_data.organisation.to_string();
    println!("Organisation: {}", organisation);
    let service = form_data.service.to_string();
    println!("Service: {}", service);
    let adresse = form_data.adresse.to_string();
    println!("Adresse Mail: {}", adresse);
    let mdp = form_data.mdp.to_string();
    println!("Mot de passe: {}", mdp);

    ma_fonction(db, form_data);

    let output = Command::new("mkdir")
        .args(&[
            format!("Serveur/User/{}", nom).as_str(),
        ])
        .output()
        .expect("Failed to execute command");

    let output = Command::new("cp")
        .args(&[
            "Serveur/CA/certs/ca.cert.pem",
            &format!("Serveur/User/{}/ca.cert.pem", nom).as_str(),
        ])
        .output()
        .expect("Failed to execute command");
        
    let output = Command::new("cp")
        .args(&[
            "Serveur/inter/certs/inter.cert.pem",
            &format!("Serveur/User/{}/inter.cert.pem", nom).as_str(),
        ])
        .output()
        .expect("Failed to execute command");

    let output = Command::new("openssl")
        .arg("genpkey")
        .arg("-algorithm")
        .arg("EC")
        .arg("-pkeyopt")
        .arg("ec_paramgen_curve:prime256v1")
        .arg("-out")
        .arg(format!("Serveur/User/{}/{}.key.pem", nom,nom))
        .output()
        .expect("La commande a échoué");
    
    let output = Command::new("openssl")
        .arg("ec")
        .arg("-in")
        .arg(format!("Serveur/User/{}/{}.key.pem",nom,nom))
        .arg("-pubout")
        .arg("-out")
        .arg(format!("Serveur/User/{}/{}.pub.pem",nom,nom))
        .output()
        .expect("La commande a échoué");
    
    let output = Command::new("openssl")
        .arg("req")
        .arg("-new")
        .arg("-key")
        .arg(format!("Serveur/User/{}/{}.key.pem", nom, nom))
        .arg("-out")
        .arg(format!("Serveur/User/{}/{}.csr.pem", nom, nom))
        .arg("-subj")
        .arg(format!(
            "/C={}/ST={}/L={}/O={}/OU={}/CN={}/emailAddress={}",
            pays,region,ville,organisation,service,nom, adresse
        ))
        .output()
        .expect("La commande a échoué");

        
    let output = Command::new("openssl")
            .arg("ca")
            .arg("-config")
            .arg("Serveur/inter/openssl.cnf")
            .arg("-extensions")
            .arg("usr_cert")
            .arg("-days")
            .arg("365")
            .arg("-notext")
            .arg("-in")
            .arg(format!("Serveur/User/{}/{}.csr.pem", nom, nom))
            .arg("-out")
            .arg(format!("Serveur/User/{}/{}.cert.pem", nom ,nom))
            .arg("-batch")
            .output()
            .expect("La commande a échoué");
        
        

    let output = Command::new("openssl")
            .arg("pkcs12")
            .arg("-export")
            .arg("-out")
            .arg(format!("Serveur/User/{}/{}_cert.p12", nom, nom))
            .arg("-inkey")
            .arg(format!("Serveur/User/{}/{}.key.pem", nom, nom))
            .arg("-in")
            .arg(format!("Serveur/User/{}/{}.cert.pem", nom, nom))
            .arg("-passout")
            .arg(format!("pass:{}", mdp))
            .output()
            .expect("La commande a échoué");
        
    

    let output = Command::new("zip")
        .arg("-r")
        .arg(format!("templates/ZIP/{}.zip", nom))
        .arg(format!("Serveur/User/{}", nom))
        .output()
        .expect("La commande a échoué");

    HttpResponse::Found()
        .header("Location", "/")
        .finish()
}



////Supprimer dans la BDD/////////////////////
async fn supp_certificat() -> io::Result<impl Responder> {
    let html = include_str!("../templates/supprimer.html");
    Ok(HttpResponse::Ok().body(html))
}

#[derive(Deserialize)]
struct Certificat2 {
    nom: String,
    prenom: String,
    code: String,
}

async fn supprimer(db: web::Data<Pool>, form_data: web::Form<Certificat2>) -> HttpResponse {
    let nom= form_data.nom.to_string();
    println!("Nom: {}", nom);
    let prenom= form_data.prenom.to_string();
    println!("Prénom: {}", prenom);
    let code= form_data.code.to_string();
    println!("Code: {}", code);
   
    let output = Command::new("openssl")
            .arg("ca")
            .arg("-config")
            .arg("Serveur/inter/openssl.cnf")
            .arg("-revoke")
            .arg(format!("Serveur/User/{}/{}.cert.pem", nom, nom))
            .output()
            .expect("La commande a échoué");
    
    let mut conn = db.get_conn().unwrap();
    conn.exec_drop(
        r"DELETE FROM utilisateurs WHERE nom = :nom AND prenom = :prenom AND code = :code",
        params! {
            "nom" => nom,
            "prenom" => prenom,
            "code" => code,
        },
    ).unwrap();

    HttpResponse::Found()
        .header("Location", "/")
        .finish()
}
/////////////////MAIN////////////////////////////

#[actix_web::main]
async fn main() -> io::Result<()> {
    let db_url = "mysql://crypto:crypto@localhost:3306/crypto";
    let db_pool = Pool::new(db_url).unwrap();
    let db_data = web::Data::new(db_pool);
    HttpServer::new(move || {
        App::new()
            .service(get_file)
            .app_data(db_data.clone())
            .route("/BDD", web::get().to(BDD))    
            .service(files)
            .service(web::resource("/").route(web::post().to(info_mail)))
            .service(web::resource("/verif.html").route(web::post().to(on_submit_form)))
            .service(web::resource("/CA.html").route(web::get().to(certificat)).route(web::post().to(create_certificat)))
            .service(web::resource("/supprimer.html").route(web::get().to(supp_certificat)).route(web::post().to(supprimer)))            
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
