CREATE TABLE rfc_problem_relations (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	rfc_id uuid NOT NULL,
	problem_id uuid NOT NULL,
	CONSTRAINT fk_rfc
		FOREIGN KEY (rfc_id)
		REFERENCES rfcs(id)
		ON DELETE CASCADE,
	CONSTRAINT fk_problem
		FOREIGN KEY (problem_id)
		REFERENCES problems(id)
		ON DELETE CASCADE
);
