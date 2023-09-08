use actix_web::{App,HttpServer,get,post,web,HttpResponse};

use askama::Template;

use serde_derive::Deserialize;

use sled::{Db,IVec};

#[derive(Template)]
#[template(path="index.html")]
struct IndexTemplate {
    data : Vec<String>,
}

#[derive(Deserialize)]
struct FormData {
    input_text: String,
}

//投稿された内容であるcontentをデータベースに書き込む関数
fn post_data(content:String)->std::io::Result<()>{

    // データベースを開く。my_dbデータベースが無い場合は生成する
    let db: Db = sled::open("my_db")?;
    
    /*
    contentをバイト文字列に変換
    sledはバイト文字列しか格納できない
    Rustのu8型は主にバイトデータやバイナリデータを表すために使用
    */
    let value_bytes = content.as_bytes();
    
    //idを生成するgenerate_id関数でidを生成し、バイト列に変換
    let id = db.generate_id().unwrap_or_default().to_be_bytes();
    
    //データベースに書き込み
    db.insert(id, value_bytes.to_vec())?;
    
    Ok(())
}

//データベースからデータをすべて取り出し、変数に格納する関数
fn get_data()->std::io::Result<Vec<String>>{

    let db: Db = sled::open("my_db")?;
    
    // データを格納するベクタを準備
    let mut value_vec: Vec<String> = Vec::new();

    // データベースからすべての値を取得してStringに変換し、ベクタに格納
    // データベースのキーとバリューを取得する(タプルになっている)
    for i in db.iter() {
        //アンラッピング
        let j: (IVec, IVec) = i.unwrap_or_default();
        
        //キーとバリューを分ける。キーは使わない
        let (_,k) =j; 
        
        //Stringに型変換 str型にしてからStringにする。
        let l =std::str::from_utf8(k.as_ref()).unwrap_or_default().to_string(); 

        //ベクタに格納
        value_vec.push(l);
    }
    
    Ok(value_vec)
}


#[get("/")]
async fn index()->HttpResponse{

    //テンプレートにデータを格納
    let html = IndexTemplate{
        data : get_data().unwrap_or_default(),
    };
    let response_body = html.render().unwrap();
    
    HttpResponse::Ok()
    .content_type("text/html")
    .body(response_body)

}


//書き込みが送信された場合
#[post("/")]
async fn handle_post(form: web::Form<FormData>) ->HttpResponse {

    if let Err(_) = post_data(form.input_text.to_string()) {
        // エラー処理を行う
        // post_data関数がエラーだった場合エラーページに遷移する
        return HttpResponse::Ok()
                .content_type("text/html")
                .body(format!("Databese Error!"))
    }

    //テンプレートにデータを格納
    let html = IndexTemplate{
        data : get_data().unwrap_or_default(),
    };
    let response_body = html.render().unwrap();
    
    HttpResponse::Ok()
    .content_type("text/html")
    .body(response_body)
}


//サーバーの名前
const SERVER_ADDR:&str = "127.0.0.1:8080";

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    println!("http://{SERVER_ADDR}");
    
    //サーバー設定
    HttpServer::new(||{
        App::new()
        .service(index)
        .service(handle_post)
    })
    .bind(SERVER_ADDR)?
    .run()
    .await
}

