# `app-demos`

This folder contains a list of more full-fledged examples (i.e., app demos) utilizing SpiderLightning/`slight`.

## `restaurant-backend`

`restaurant-backend` is a `slight` app that simulates the operation of a restaurant, where people can order food through a HTTP `POST` request to the `/orders/` route, and a chef can get the next order to prepare through a HTTP `GET` request to the `/orders/next/` route.

Assuming you've got the latest version of `slight` compiled (i.e., by having ran `make build`), from the root of the repository, run `make build-app-demos` to compile all app demos. After that, run `make run-restaurant-backend` to start the HTTP server that will listen to requests — you should see something like so:

![](https://i.imgur.com/4etNeMm.png)

Next up, you can test this service with `curl`. Let's make a HTTP `POST` request like so:
```shell
curl http://localhost:3000/orders/ -d burger
```

Looking back on the server output, you should see:
![](https://i.imgur.com/rkadRSp.png)

To finish off, let's make a HTTP `GET` request to receive that burger order:
```shell
curl http://localhost:3000/orders/next
```

On your shell, you should see:
```shell
burger
```

On the server output, you should see:
![](https://i.imgur.com/OcZlrsD.png)