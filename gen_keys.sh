#/bin/sh
if [ "$#" -ne 1 ]
then
  echo "Error: No domain name argument provided"
  echo "Usage: Provide a domain name as an argument"
  exit 1
fi

DOMAIN=$1
CERTS_PATH=certs

mkdir -p ${CERTS_PATH}

# Create root CA & Private key

openssl req -x509 \
            -sha256 -days 356 \
            -nodes \
            -newkey rsa:2048 \
            -subj "/CN=${DOMAIN}/C=US/L=San Fransisco" \
            -keyout ${CERTS_PATH}/rootCA.key -out ${CERTS_PATH}/rootCA.crt 

# Generate Private key 

openssl genrsa -out ${CERTS_PATH}/key.pem 2048

# Create csf conf

cat > ${CERTS_PATH}/csr.conf <<EOF
[ req ]
default_bits = 2048
prompt = no
default_md = sha256
req_extensions = req_ext
distinguished_name = dn

[ dn ]
C = US
ST = Virginia
L = Fairfax
O = CERG
OU = CERG
CN = ${DOMAIN}

[ req_ext ]
subjectAltName = @alt_names

[ alt_names ]
DNS.1 = ${DOMAIN}
DNS.2 = www.${DOMAIN}
DNS.3 = localhost
IP.1 = 0.0.0.0 
IP.2 = 127.0.0.1

EOF

# create CSR request using private key

openssl req -new -key ${CERTS_PATH}/key.pem -out ${CERTS_PATH}/${DOMAIN}.csr -config ${CERTS_PATH}/csr.conf

# Create a external config file for the certificate

cat > ${CERTS_PATH}/cert.conf <<EOF

authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = ${DOMAIN}

EOF

# Create SSl with self signed CA

openssl x509 -req \
    -in ${CERTS_PATH}/${DOMAIN}.csr \
    -CA ${CERTS_PATH}/rootCA.crt -CAkey ${CERTS_PATH}/rootCA.key \
    -CAcreateserial -out ${CERTS_PATH}/cert.pem \
    -days 365 \
    -sha256 -extfile ${CERTS_PATH}/cert.conf