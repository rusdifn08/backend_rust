CREATE TABLE avatars (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    url VARCHAR(500) NOT NULL
);

INSERT INTO avatars (name, url) VALUES
('Boy 1', 'https://i.pravatar.cc/150?img=11'),
('Boy 2', 'https://i.pravatar.cc/150?img=12'),
('Girl 1', 'https://i.pravatar.cc/150?img=10'),
('Girl 2', 'https://i.pravatar.cc/150?img=9'),
('Man 1', 'https://i.pravatar.cc/150?img=15'),
('Man 2', 'https://i.pravatar.cc/150?img=17'),
('Woman 1', 'https://i.pravatar.cc/150?img=20'),
('Woman 2', 'https://i.pravatar.cc/150?img=22'),
('Cartoon Cat', 'https://i.pravatar.cc/150?img=25'),
('Anime Boy', 'https://i.pravatar.cc/150?img=33'),
('Anime Girl', 'https://i.pravatar.cc/150?img=31'),
('Business Man', 'https://i.pravatar.cc/150?img=50'),
('Business Woman', 'https://i.pravatar.cc/150?img=49'),
('Creative 1', 'https://i.pravatar.cc/150?img=60'),
('Creative 2', 'https://i.pravatar.cc/150?img=65');
