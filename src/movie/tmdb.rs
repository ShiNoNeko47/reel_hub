use super::MovieData;

pub fn fetch_data_tmdb(name: &String, year: String) -> Option<MovieData> {
    match reqwest::blocking::get(format!(
        "https://api.themoviedb.org/3/search/movie?query={}{}&api_key={}",
        name, year, "f090bb54758cabf231fb605d3e3e0468"
    )) {
        Ok(response) => {
            let data = response.text().unwrap().to_string();
            let results: serde_json::Value = serde_json::from_str(&data).unwrap();
            let mut movie_data = &results["results"][0];
            for result in results["results"].as_array().unwrap() {
                let title = result["title"].as_str().unwrap().to_string();
                let release_date = result["release_date"].as_str().unwrap().to_string();
                if title == name.to_string() && release_date.contains(&year.replace("&year=", "")) {
                    movie_data = result;
                    break;
                }
            }
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
            })
        }
        _ => None,
    };
    None
}
