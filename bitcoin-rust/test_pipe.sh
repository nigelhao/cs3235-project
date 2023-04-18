# Demonstrate the usage of named pipe
mkfifo testpipe
python3 ./tests/_bots/botA-1.py > testpipe &
timeout 5 cat testpipe
/bin/rm -f testpipe