curl -X GET http://localhost:8080/application -v -4 -i \
     -H 'Content-Type: application/json' \
     -H 'Accept: application/json' \
     -H "Authorization: Bearer ${SESSION_JWT}"
