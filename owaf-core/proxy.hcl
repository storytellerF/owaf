proxy "minio" {
  host = "minio.example.com"
  target = "${MINIO_URL}"
}

proxy "mysql" {
  host = "mysql.example.com"
  target = "${MYSQL_URL}"
}
