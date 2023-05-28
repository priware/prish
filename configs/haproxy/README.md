# HAProxy mTLS
In order to have secure access to your server via `prish` you should
use mTLS to authenticate the clients.

## Create a CA
Here is the steps to create a Root CA, Intermediate CA and
certificates for client and server.

```bash
openssl ecparam -name secp384r1 > ec.param

openssl req -new -x509 -nodes -newkey ec:ec.param -keyout root-ca.key -out root-ca.crt -days 3650

openssl req -nodes -newkey ec:ec.param -keyout intermediate-ca.key -out intermediate-ca.csr -days 3650

openssl x509 -req -in intermediate-ca.csr -out intermediate-ca.crt \
    -CA root-ca.crt -CAkey root-ca.key -CAcreateserial \
    -days 3650 -extfile ca-cert-extensions.cnf
    
openssl req  -nodes -newkey ec:ec.param -keyout client.key -out client.csr -days 365

openssl x509 -req -in client.csr -out client.crt -CA intermediate-ca.crt -CAkey intermediate-ca.key \
    -CAcreateserial -days 365 -extfile client-cert-extensions.cnf

openssl req  -nodes -newkey ec:ec.param -keyout server.key -out server.csr -days 365

openssl x509 -req -in server.csr -out server.crt -CA intermediate-ca.crt -CAkey intermediate-ca.key \
    -CAcreateserial -days 365 -extfile server-cert-extensions.cnf
    
openssl pkcs12 -export -inkey client.key -in client.crt -out client.p12

cat server.crt server.key > server.pem
```

## Configure HAProxy
Install HAProxy and run the following commands:

```bash
mkdir /etc/haproxy/certs/
cp server.pem /etc/haproxy/certs/
cp intermediate-ca.crt /etc/haproxy/certs/
cp root-ca.crt /etc/haproxy/certs/
```

Append following to the `/etc/haproxy/haproxy.cfg` and restart haproxy.

```conf
frontend prish-front
  bind 0.0.0.0:80
  bind 0.0.0.0:443  ssl crt /etc/haproxy/certs/server.pem verify required ca-file /etc/haproxy/certs/intermediate-ca.crt ca-verify-file /etc/haproxy/certs/root-ca.crt
  http-request redirect scheme https unless { ssl_fc }
  default_backend prish

backend prish
  mode http
  server s0 127.0.0.1:3030
```

### Configure Chrome
Go to the `Settings > Privacy and security > Security > Manage certififcats > Your certificates` and import `client.p12`.
