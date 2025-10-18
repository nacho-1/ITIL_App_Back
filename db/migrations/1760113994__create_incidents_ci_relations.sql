CREATE TABLE incidents_ci_relations (
	incident_id uuid NOT NULL,
	ci_id uuid NOT NULL,
	PRIMARY KEY (incident_id, ci_id),
	description TEXT NOT NULL,
	CONSTRAINT fk_incident
		FOREIGN KEY (incident_id)
		REFERENCES incidents(id)
		ON DELETE CASCADE,
	CONSTRAINT fk_ci
		FOREIGN KEY (ci_id)
		REFERENCES configitems(id)
		ON DELETE CASCADE
);
