CREATE TABLE rfc_incident_relations (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	rfc_id uuid NOT NULL,
	incident_id uuid NOT NULL,
	CONSTRAINT fk_rfc
		FOREIGN KEY (rfc_id)
		REFERENCES rfcs(id)
		ON DELETE CASCADE,
	CONSTRAINT fk_incident
		FOREIGN KEY (incident_id)
		REFERENCES incidents(id)
		ON DELETE CASCADE
);
