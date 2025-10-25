CREATE TABLE problem_incident_relations (
	problem_id uuid NOT NULL,
	incident_id uuid NOT NULL,
	PRIMARY KEY (problem_id, incident_id),
	description TEXT NOT NULL,
	CONSTRAINT fk_problem
		FOREIGN KEY (problem_id)
		REFERENCES problems(id)
		ON DELETE CASCADE,
	CONSTRAINT fk_incident
		FOREIGN KEY (incident_id)
		REFERENCES incidents(id)
		ON DELETE CASCADE
);
