xset r rate 150 30
docker run --rm --name cli-pong -it cli:latest ./cli_app $1
xset r rate 660 25
echo "Thank you for playing"