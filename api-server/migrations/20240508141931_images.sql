CREATE TABLE IF NOT EXISTS Images (
    image_id INTEGER PRIMARY KEY,
    loc INTEGER,
    encoded_image TEXT,
    FOREIGN KEY (loc) REFERENCES Locations (location_id)
);