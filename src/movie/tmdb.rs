use super::MovieData;

pub fn fetch_data_tmdb(name: &String, year: String) -> Option<MovieData> {
    let addr = format!("https://api.themoviedb.org/3/search/movie?query={}{}&api_key=f090bb54758cabf231fb605d3e3e0468", name, year);
    match reqwest::blocking::get(addr) {
        Ok(response) => {
            let data: String = response.text().unwrap().to_string();
            let results: serde_json::Value = serde_json::from_str(&data).unwrap();
            let mut movie_data: &serde_json::Value = &results["results"][0];
            for result in results["results"].as_array().unwrap() {
                let title: String = result["title"].as_str().unwrap().to_string();
                let release_date: String = result["release_date"].as_str().unwrap().to_string();
                if title == name.to_string() && release_date.contains(&year.replace("&year=", "")) {
                    movie_data = result;
                    break;
                }
            }
            if movie_data != &serde_json::Value::Null {
                Some(MovieData {
                    title: movie_data["title"].as_str().unwrap().to_string(),
                    original_title: movie_data["original_title"].as_str().unwrap().to_string(),
                    original_language: movie_data["original_language"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                    overview: movie_data["overview"].as_str().unwrap().to_string(),
                    vote_average: movie_data["vote_average"].as_f64().unwrap(),
                    vote_count: movie_data["vote_count"].as_u64().unwrap(),
                    release_date: movie_data["release_date"].as_str().unwrap().to_string(),
                    poster_path: movie_data["poster_path"].as_str().unwrap().to_string(),
                    backdrop_path: movie_data["backdrop_path"].as_str().unwrap().to_string(),
                })
            } else {
                Some(MovieData {
                    title: name.to_string(),
                    original_title: "".to_string(),
                    original_language: "".to_string(),
                    overview: "".to_string(),
                    vote_average: 0.0,
                    vote_count: 0,
                    release_date: "".to_string(),
                    poster_path: "".to_string(),
                    backdrop_path: "".to_string(),
                })
            }
        }
        _ => None,
    }
}

pub fn fetch_image_tmdb(image_path: String, resolution: Option<u32>) -> Vec<u8> {
    let resolution = match resolution {
        Some(resolution) => resolution,
        None => 185,
    };

    let result = reqwest::blocking::get(format!(
        "https://image.tmdb.org/t/p/w{resolution}/{image_path}"
    ))
    .unwrap()
    .bytes()
    .unwrap()
    .to_vec();
    result
}
