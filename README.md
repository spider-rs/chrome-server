# chrome

Chrome cross compiled dockerfile with tini startup

## Features

1. Can spawn multiple chrome instances.
1. Get chrome ws connections and status.


The current instance binds chrome to 0.0.0.0 when starting via API.

Use the env variable `REMOTE_ADDRESS` to change the address of the chrome instance between physical or network.

## Todo

1. Control chrome instances via http to spin up and down.
2. opt to build chrome direct without docker.