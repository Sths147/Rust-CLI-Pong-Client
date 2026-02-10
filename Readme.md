# Cli-app
### This application is designed to connect to a [transcendence server](https://github.com/Maxime-juncker/ft_transcendence)
Running this tool enables you to connect to the Pong website and play Pong on your terminal
## Warning: 
### Running the app modifies the delay of appearance of the Repeat Event of your keyboard, so close your app when you're done playing
Indeed, in order to improve playability of the game, we modify the settings of the keyboard events while the game is played. 
Be sure to quit the game once you are done. The settings restored back are: 660ms delay for KeyRepeat and 25ms delay between repeats
you can adjust those settings with the command

```bash
xset r rate [firstRepeatDelay] [betweenRepeatDelay]
```

Users using Wayland won't be impacted by these settings

## Requirements:

docker

git

## Installation:

You need first to install the repo by running: 

```bash
git clone https://github.com/Sths147/Rust-CLI-Pong-Client.git cli-transcendence
```

First time: Execute:

```bash
./build.sh
``` 
to build the docker image

Then execute:

```bash
./launch.sh $LOCATION [R_REPEAT] [R_RATE]
```
while replacing $LOCATION with the IP:PORT the transcendence server listens to. Options R_REPEAT and R_RATE can be set to adjust KeyRepeat delays 

Note: This application runs on a Docker container, so if the game server runs on the same machine as the cli app, running
```bash
./launch localhost:XXXX
```
will not work. You can replace IP with the docker0 bridge address or your IP address. You can get required information with the following command:
```bash
ip address
``` 

## How to play

During the whole game, a menu will be displayed to indicate different options
Generally, arrows will be used to navigate and to play, ESC and Ctrl+C will quit or go back.
Enjoy the game!