CREATE DATABASE projects;
CREATE USER 'jaykchen'@'localhost' IDENTIFIED BY 'Sunday228';
GRANT ALL PRIVILEGES ON projects.* TO 'jaykchen'@'localhost';
FLUSH PRIVILEGES;
EXIT;
