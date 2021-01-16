# rust_iot_api
Rust dilinde yazılmış sıcaklık ve nem kaydını tutan basit bir API.

API 8000 portta çalışır. Veri kaydetmek için 

`curl -H "Content-Type: application/json" -X POST -d {\"sicaklik\":7.56,\"nem\":2.46} http://localhost:8000/sensor`'

formatında sıcaklık ve nem bilgilerini gönderin. 127.0.0.1:8000 adresine `GET` isteği atarsanız tüm kayıtlı veriler veri tabanından getirilir. 