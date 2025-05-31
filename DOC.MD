# Expresiones
- `@(value)` convierte a iterador.
- `?(value)` convierte el valor a un booleano.
- `!(value) == !?(value)` niega el valor booleano.
- `&(value)` convierte el valor a inmutable.
- `+(value)` convierte el valor a número.
- `-(value)` convierte el valor a número negativo.
- `~(value)` convierte el valor a número entero.
- `(value)?` si el valor es un error (lanzado), se convierte en nulo. Ejemplo: `1/{}?` es nulo.
- `(value1) && (value2)` retorna el primer valor falso o el último verdadero.
- `(value1) || (value2)` retorna el primer valor verdadero o el último falso.
- `(value1) ?? (value2)` si `value1` es nulo, retorna `value2`.
- `(value1) = (value2)` asigna `value2` a `value1`.
- `(value1) == (value2)` verifica si ambos valores son iguales.
- `(value1) != (value2)` verifica si ambos valores son diferentes.
- `(value1) >= (value2)` verifica si `value1` es mayor o igual que `value2`.
- `(value1) <= (value2)` verifica si `value1` es menor o igual que `value2`.
- `(value1) > (value2)` verifica si `value1` es mayor que `value2`.
- `(value1) < (value2)` verifica si `value1` es menor que `value2`.
- `(value1) + (value2)` suma `value1` y `value2`.
- `(value1) - (value2)` resta `value1` y `value2`.
- `(value1) * (value2)` multiplica `value1` y `value2`.
- `(value1) / (value2)` divide `value1` entre `value2`.
- `(value1) % (value2)` retorna el residuo de la división.
- `(value1) ^ (value2)` eleva `value1` a la potencia de `value2`.
- `(value1) &&= (value2)` si `value1` es verdadero, asigna `value2`.
- `(value1) ||= (value2)` si `value1` es falso, asigna `value2`.
- `(value1) ??= (value2)` si `value1` es nulo, asigna `value2`.
- `(value1) += (value2)` suma `value2` a `value1` y asigna el resultado.
- `(value1) -= (value2)` resta `value2` de `value1` y asigna el resultado.
- `(value1) *= (value2)` multiplica `value1` por `value2` y asigna el resultado.
- `(value1) /= (value2)` divide `value1` entre `value2` y asigna el resultado.
- `(value1) %= (value2)` obtiene el residuo de la división y asigna el resultado.
- `(value1) ^= (value2)` eleva `value1` a `value2` y asigna el resultado.

# Tipos de Datos
## Primitivos
### Booleano

### Byte

### Número
- `(numero)` recibe un número como argumento para ser multiplicado.
    - `numero`: [Número](#numero) a multiplicar.
    - Retorna un [Número](#numero).

### Cadena
- `reemplaza(viejo, nuevo)` reemplaza el texto `viejo` por el `nuevo`.
    - `viejo`: [Cadena](#cadena).
    - `nuevo`: [Cadena](#cadena).
    - Retorna una [Cadena](#cadena).
- `bytes()` convierte la cadena a una lista de bytes.
    - Retorna un [Buffer](#buffer).
- `separa(cortador)` divide la cadena cada vez que el texto coincide con el `cortador`.
    - `cortador`: [Cadena](#cadena).
    - Retorna una [Lista](#lista) de [Cadenas](#cadena).
- `longitud` retorna un [Número](#numero).
- `[indice]` retorna un [Carácter](#caracter).

#### Carácter

## Complejos
### Lista
- `[indice]` retorna un valor cualquiera.

### Clase
Las clases solo pueden extender sus métodos estáticos.

### Función
Extiende de [Función Nativa](#funcion-nativa).

### Objeto
Extiende las propiedades de instancia de la clase que lo creó.

### Promesa
- `luego(funcion)` recibe una función que tiene como argumento el resultado de la promesa en caso de éxito y retorna una nueva promesa.
    - `funcion(valor)`: 
      - `valor`: el valor que fue resuelto.
      - Retorna una [promesa](#promesa).
- `atrapa(funcion)` recibe una función que tiene como argumento el lanzamiento de la promesa en caso de fallo y retorna una nueva promesa.
    - `funcion(valor)`: 
      - `valor`: el valor que fue [lanzado](#lanzado).
      - Retorna una [promesa](#promesa).

## Internos
### Error
- `tipo`: [Cadena](#cadena).
- `mensaje`: [Cadena](#cadena).

### Inmutable
Hereda todos los métodos del valor que envuelve.

### Vago
Espera a ser llamado para obtener su valor.

### Lanzado
Este valor debe ser atrapado para no detener el programa.

### Función Nativa
- `llamar(este, argumentos)` llama a la función como si el primer parámetro fuera el dueño de la función.
    - `este`: valor cualquiera.
    - `argumentos`: [Lista](#lista) de valores cualquiera.
- `(...argumentos)` puede ser llamada con los argumentos en una [Lista](#lista).

### Buffer
Extiende de [Lista](#lista).
- `[indice]` solo puede contener [Bytes](#byte).

# Librerías
## Sistema de Archivos
`sa` implementa funciones para el manejo de archivos.
- `leer_archivo(ruta)` recibe la ruta de un archivo y regresa una lista de bytes.
    - `ruta`: [Cadena](#cadena).
    - Responde con un [Buffer](#buffer).
- `leer_carpeta(ruta)` recibe la ruta de una carpeta y regresa una lista con los nombres de los archivos/carpetas dentro.
    - `ruta`: [Cadena](#cadena).
    - Responde con una [Lista](#lista).
- `obtener_ruta(ruta)` recibe una ruta y regresa una instancia de [Ruta](#ruta).
    - `ruta`: [Cadena](#cadena).
    - Responde con una [Ruta](#ruta).
- `escribir_archivo(ruta, buffer)` escribe el buffer a partir del último byte.
    - `buffer`: [Buffer](#buffer).
    - `ruta`: [Cadena](#cadena).
- `crear_archivo(ruta, buffer)` crea o sobreescribe el archivo con el buffer.
    - `buffer`: [Buffer](#buffer).
    - `ruta`: [Cadena](#cadena).
- `crear_carpeta(ruta)` crea una carpeta si está disponible.
    - `ruta`: [Cadena](#cadena).

### Clases
#### Ruta
- `es_archivo()` retorna un [Booleano](#buleano).
- `es_carpeta()` retorna un [Booleano](#buleano).
- `nombre()` retorna una [Cadena](#cadena).
- `obtener_padre()` retorna una [Cadena](#cadena).

## Matemáticas
`mate` proporciona funciones matemáticas básicas.
- `suelo(numero)` recibe un número y quita los decimales.
    - `numero`: [Número](#numero).
    - Responde con un [Número](#numero).
- `min(...numeros)` retorna el número más pequeño de la lista.
    - `numeros`: [Lista](#lista) de [Números](#numero).
    - Responde con un [Número](#numero).
- `max(...numeros)` retorna el número más grande de la lista.
    - `numeros`: [Lista](#lista) de [Números](#numero).
    - Responde con un [Número](#numero).

## Tiempo
`temp` proporciona funciones para el manejo de tiempo.
- `esperar(segundos)` retorna una promesa que se resuelve después de `segundos`.
    - `segundos`: cantidad de segundos como [Número](#numero).

## Consola
`consola` proporciona complementos para mostrar información en la consola.
- `pintar(...argumentos)` imprime los `argumentos`.
    - `argumentos`: [Lista](#lista) de valores cualesquiera.

## Red
`red` maneja implementaciones en red.
- `servidorTCP(host, funcion)` crea un servidor TCP.
    - `host`: dirección de acceso, por ejemplo, `128.0.0.1:80`.
    - `funcion(peticion)`: función que recibe la [Petición](#buffer) como argumento.
        - `peticion`: [Buffer](#buffer).
        - Responde con un [Buffer](#buffer).
    - Responde con una [Promesa](#promesa).
