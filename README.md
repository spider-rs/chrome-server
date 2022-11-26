# chrome

Chrome instance and panel

## Usage

1. Can spawn multiple chrome instances.
1. Get chrome ws connections and status.

The current instance binds chrome to 0.0.0.0 when starting via API.

Use the env variable `REMOTE_ADDRESS` to change the address of the chrome instance between physical or network.

The application will pass alp health checks when using port `6000` to get the status of the chrome container.

A side loaded application is required to run chrome on a load balancer, one of the main purposes of the control panel.

## Todo

1. Control chrome instances via http to spin up and down.
2. opt to build chrome direct without docker.