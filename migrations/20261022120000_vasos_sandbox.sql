-- Geometría de los vasos tomada del banco de pruebas (ui-ux/sandbox).
-- Los vasos del catálogo se habían calculado para que su volumen coincidiera con la
-- capacidad declarada, y eso los dejaba anchos y bajos: la taza de 160 ml salía como un
-- bol de 84 × 42. El sandbox usa proporciones dibujadas a mano (las mismas que el
-- prototipo), y es lo que hace que la taza parezca una taza. La capacidad declarada no
-- cambia: el motor solo la usa para repartir el volumen entre las capas.

-- Taza de espresso (90 ml) → demitasse
UPDATE vaso
SET forma = 'taza',
    radio_boca = 46,
    radio_fondo = 34,
    altura = 76,
    borde = 4,
    pared = 4,
    tallo = 0,
    asa = 'chica',
    plato = true
WHERE clave = 'taza_espresso';

-- Taza de 160 ml → mug
UPDATE vaso
SET forma = 'taza',
    radio_boca = 62,
    radio_fondo = 52,
    altura = 128,
    borde = 5,
    pared = 5,
    tallo = 0,
    asa = 'grande',
    plato = true
WHERE clave = 'taza_160';

-- Gibraltar (135 ml) → vaso bajo
UPDATE vaso
SET forma = 'vaso',
    radio_boca = 42,
    radio_fondo = 39,
    altura = 98,
    borde = 5,
    pared = 5,
    tallo = 0,
    asa = NULL,
    plato = false
WHERE clave = 'vaso_gibraltar';

-- Vaso de latte (300 ml) → vaso alto
UPDATE vaso
SET forma = 'vaso',
    radio_boca = 43,
    radio_fondo = 38,
    altura = 162,
    borde = 6,
    pared = 4,
    tallo = 0,
    asa = NULL,
    plato = false
WHERE clave = 'vaso_latte';

-- Copa de cóctel (180 ml) → copa con tallo
UPDATE vaso
SET forma = 'copa',
    radio_boca = 47,
    radio_fondo = 4,
    altura = 76,
    borde = 4,
    pared = 2.5,
    tallo = 66,
    asa = NULL,
    plato = false
WHERE clave = 'copa_coctel';

-- Vaso rocks (300 ml) → old fashioned
UPDATE vaso
SET forma = 'vaso',
    radio_boca = 42,
    radio_fondo = 39,
    altura = 98,
    borde = 5,
    pared = 5,
    tallo = 0,
    asa = NULL,
    plato = false
WHERE clave = 'vaso_rocks';

-- Highball (350 ml) → vaso alto y estrecho
UPDATE vaso
SET forma = 'vaso',
    radio_boca = 33,
    radio_fondo = 30,
    altura = 186,
    borde = 6,
    pared = 4,
    tallo = 0,
    asa = NULL,
    plato = false
WHERE clave = 'vaso_highball';
