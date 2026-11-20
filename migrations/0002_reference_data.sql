-- Reference data that every environment needs: faculties and specialties of TU-Sofia used in the demo.
INSERT INTO faculties (code, name) VALUES
('FCST', 'Факултет по компютърни системи и технологии'),
('FAMI', 'Факултет по приложна математика и информатика'),
('FE',   'Електротехнически факултет');

INSERT INTO specialties (faculty_code, name) VALUES
('FCST', 'Компютърни науки'),
('FCST', 'Софтуерно инженерство'),
('FCST', 'Компютърно и софтуерно инженерство'),
('FAMI', 'Информатика'),
('FAMI', 'Приложна математика'),
('FE',   'Електроенергетика');
