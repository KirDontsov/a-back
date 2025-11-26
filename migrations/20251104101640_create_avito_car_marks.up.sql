-- Create avito_car_marks table
CREATE TABLE avito_car_marks (
    car_mark_id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    value VARCHAR(255) NOT NULL
);

-- Insert some sample data with explicit UUIDs
INSERT INTO avito_car_marks (car_mark_id, value) VALUES
('11111111-1111-1111-1111-11111111', 'Toyota'),
('22222222-2222-2222-2222-22222222', 'Honda'),
('3333-3333-3333-3333-3333', 'Ford'),
('44444444-4444-4444-4444-44444444', 'BMW'),
('55555555-5555-5555-5555-55555555', 'Mercedes-Benz');