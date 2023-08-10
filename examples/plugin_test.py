#! /usr/bin/env python3


def setup():
    """
    setup the environment
    e.g. download images, fetch any data needed by the plugin...
    """
    pass


def handle_request(plugin_id: int, request: str):
    if request == "add":
        """
         - only "movie", name and source are required
         - if there is nothing after done, other data is fetched from tmdb
         - semicolon (;) is used to separate data
         - to make reel_hub fetch data from tmdb,
           nothing should be written after done

         - minimum: movie;name;;source


        format:
            "movie" -> tells reel_hub to add a movie with following data
            name -> name to be displayed on the button
            year -> optional, used to narrow down results from tmdb
            source
            current_time
            duration
            done -> whether the movie has already been watched

            title
            original_title
            overview
            original_language
            poster_path
            backdrop_path
            vote_average
            vote_count
            release_date
            genre_ids -> takes everything after release_date
                         ids are seperated by semicolon (;)

        see examples below
        """

        print(
            "movie;HRT1;;https://webtvstream.bhtelecom.ba/hls9/hrt1_1200.m3u8;;;;HRT1;HRT1;Hrvatska radio televizija;hr;/hrt1.png;;;;;tv"
        )
        print(
            "movie;RTL;;https://d1cs5tlhj75jxe.cloudfront.net/rtl/playlist.m3u8;;;;RTL;RTL"
        )
        print(
            plugin_id
            + ";movie;Ringu;1998;https://www.youtube.com/watch?v=CQ1jkNj4cZc;;5716;"
        )
        while True:
            response = input().split(";")
            if response[0] == "movie_id":
                break
            handle_request(plugin_id, ";".join(response))
        print(f"Ringu_id;{response[1]}")


def main():
    setup()
    plugin_id = input().split(";")[1]
    while True:
        request = input()
        if request == "0":
            break
        handle_request(plugin_id, request)


if __name__ == "__main__":
    main()
