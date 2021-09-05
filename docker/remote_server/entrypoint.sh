
#!/bin/bash

service postgresql start
service nginx restart

sleep 5

cargo run & 

/bin/bash