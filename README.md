## reel_hub
This is a gtk movie library browser written in rust. 

## Features
 - Fetching data from <b>tmdb api</b> 
	- poster
	- backdrop
	- original title and language
	- overview
	- vote average and count
	- release date
 - Playing the selected movie in <b>mpv</b>
 - Tracking progress

## Installation
### Arch linux

Use your favourite package manager to install from AUR

### Manual
Clone this repository first:
```sh
git clone https://github.com/ShiNoNeko47/reel_hub
```
---
If you just want to run the app:
```sh
cd reel_hub && cargo run -r
```
---
To make it accessible your from application menu, first compile binary using
```sh
cd reel_hub && cargo build -r
```
then copy the desktop file to ```XDG_DATA_HOME/applications/reel_hub.desktop```
and <b>either</b>
replace ```Exec = reel_hub```
with ```Exec = FULL_PATH_TO_BINARY``` eg. ```Exec = /home/user/reel_hub/target/reel_hub```
<b>or</b>
place the binary on path 

## Usage
```
Usage: reel_hub [option]

 -v, --version		show varsion and exit
 -h, --help		show this help and exit
 -l, --list		list all movies in library and exit
 -c, --clear-cache	clear cache and exit (does not clear time positions)
```
<b>Note:</b> running without arguments runs the app
## Screenshots
<b>None:</b> backdrop gets hidden when there isn't enough space

![image](screenshot_1.png)
![image](screenshot_2.png)
