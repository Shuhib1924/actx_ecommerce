use actix_web::{HttpResponse, Result};

pub async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body(r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Rust E-Commerce</title>
            <script src="https://cdn.tailwindcss.com"></script>
        </head>
        <body class="bg-gray-100">
            <div class="container mx-auto px-4 py-8">
                <h1 class="text-4xl font-bold text-center text-gray-800">
                    Welcome to Rust E-Commerce
                </h1>
                <p class="text-center mt-4 text-gray-600">
                    Your shop is being set up with SQLite...
                </p>
                <div class="mt-8 flex justify-center space-x-4">
                    <a href="/store" class="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600">
                        Browse Store
                    </a>
                    <a href="/admin" class="bg-green-500 text-white px-4 py-2 rounded hover:bg-green-600">
                        Admin Panel
                    </a>
                </div>
            </div>
        </body>
        </html>
    "#))
}