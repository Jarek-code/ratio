-- Versiones del Cappuccino: dos derivadas de la receta original.
--
-- El historial (B6) se apoya en `receta.receta_origen_id`, que hasta ahora estaba vacío en
-- todas las filas. Se añaden dos variantes con cambios legibles ingrediente a ingrediente:
-- una publicada y otra privada, para que el filtro por visibilidad tenga contenido.
--
-- Las variantes se atribuyen a la cuenta con perfil más reciente (la de pruebas) para que la
-- privada se vea con su sesión; si no hay ninguna, heredan el autor del original.
-- Las fechas se calculan desde la de la receta original para que la línea de tiempo quede en
-- orden: primero el original, después sus variantes. Es idempotente: cada bloque solo inserta
-- si esa versión no existe todavía.

-- Cappuccino seco: la misma leche con menos agua, más espuma.
insert into receta (autor_id, receta_origen_id, nombre, tipo, vaso_id, capacidad_ml, familia,
                    tecnica, temperatura, dificultad, intensidad, minutos, notas, etiquetas,
                    publicada_en, creada_en)
select coalesce((select u.id from usuario u order by u.creado_en desc limit 1), r.autor_id), r.id, 'Cappuccino seco', r.tipo, r.vaso_id, r.capacidad_ml, r.familia,
       r.tecnica, r.temperatura, r.dificultad, r.intensidad, r.minutos, r.notas, r.etiquetas,
       r.creada_en + interval '4 hours', r.creada_en + interval '4 hours'
from receta r
where r.nombre = 'Cappuccino'
  and not exists (select 1 from receta x where x.nombre = 'Cappuccino seco');

-- Cappuccino doble: dos espressos en la misma taza, más leche para equilibrar.
insert into receta (autor_id, receta_origen_id, nombre, tipo, vaso_id, capacidad_ml, familia,
                    tecnica, temperatura, dificultad, intensidad, minutos, notas, etiquetas,
                    publicada_en, oculta, creada_en)
select coalesce((select u.id from usuario u order by u.creado_en desc limit 1), r.autor_id), r.id, 'Cappuccino doble', r.tipo, r.vaso_id, r.capacidad_ml, r.familia,
       r.tecnica, r.temperatura, r.dificultad, r.intensidad, r.minutos, r.notas, r.etiquetas,
       null, false, r.creada_en + interval '1 day'
from receta r
where r.nombre = 'Cappuccino'
  and not exists (select 1 from receta x where x.nombre = 'Cappuccino doble');

-- Componentes del Cappuccino seco: menos leche vaporizada, más espuma, mismo espresso.
insert into componente_receta (receta_id, posicion, ingrediente_id, nombre, graduacion_pct,
                               color, cantidad, unidad, dilucion)
select n.id, c.posicion, c.ingrediente_id, c.nombre, c.graduacion_pct, c.color,
       case c.nombre
           when 'Leche vaporizada' then 40
           when 'Espuma de leche' then 85
           else c.cantidad
       end,
       c.unidad, c.dilucion
from receta n
join receta r on r.nombre = 'Cappuccino' and n.receta_origen_id = r.id
join componente_receta c on c.receta_id = r.id
where n.nombre = 'Cappuccino seco'
  and not exists (select 1 from componente_receta x where x.receta_id = n.id);

-- Componentes del Cappuccino doble: espresso y leche al doble, misma espuma.
insert into componente_receta (receta_id, posicion, ingrediente_id, nombre, graduacion_pct,
                               color, cantidad, unidad, dilucion)
select n.id, c.posicion, c.ingrediente_id, c.nombre, c.graduacion_pct, c.color,
       case c.nombre
           when 'Espresso' then c.cantidad * 2
           when 'Leche vaporizada' then 70
           else c.cantidad
       end,
       c.unidad, c.dilucion
from receta n
join receta r on r.nombre = 'Cappuccino' and n.receta_origen_id = r.id
join componente_receta c on c.receta_id = r.id
where n.nombre = 'Cappuccino doble'
  and not exists (select 1 from componente_receta x where x.receta_id = n.id);

-- Los pasos se copian del original para que la ficha de cada versión no quede vacía.
insert into paso (receta_id, posicion, instruccion, temporizador_s, titulo)
select n.id, p.posicion, p.instruccion, p.temporizador_s, p.titulo
from receta n
join receta r on r.nombre = 'Cappuccino' and n.receta_origen_id = r.id
join paso p on p.receta_id = r.id
where n.nombre in ('Cappuccino seco', 'Cappuccino doble')
  and not exists (select 1 from paso x where x.receta_id = n.id);
