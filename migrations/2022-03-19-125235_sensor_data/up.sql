-- Your SQL goes here
CREATE TABLE sensor_data (
	id SERIAL NOT NULL, 
	nem FLOAT NOT NULL, 
	sicaklik FLOAT NOT NULL, 
	timestamp TIMESTAMP NOT NULL, 
	PRIMARY KEY (id)
)