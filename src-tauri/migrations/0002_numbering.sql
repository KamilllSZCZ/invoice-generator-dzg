ALTER TABLE numbering_series ADD COLUMN reset_policy TEXT NOT NULL DEFAULT 'never';
UPDATE numbering_series SET reset_policy = CASE WHEN counter_period IN ('yearly', 'monthly', 'never') THEN counter_period ELSE 'never' END;
UPDATE numbering_series SET counter_period = NULL;
UPDATE numbering_series SET next_counter = 1 WHERE next_counter < 1;
