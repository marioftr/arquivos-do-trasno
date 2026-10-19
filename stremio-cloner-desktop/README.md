# Stremio Account Addon Cloner — versión de escritorio

Empaqueta [stremio-account-addon-cloner](https://github.com/oozmakafa/stremio-account-addon-cloner)
(Next.js con rutas API de servidor) como ejecutable con Electron. El ejecutable arranca el servidor
Next.js en local (puerto libre, solo 127.0.0.1) y lo muestra en una ventana.

## Requisitos
Node.js >= 18 y Git. Hay que compilar **en el sistema operativo de destino**.

## Generar el ejecutable
```
npm install
npm run dist
```
El resultado queda en `dist/`:
- Windows: `Stremio Addon Cloner x.y.z.exe` (portable) y un instalador NSIS
- macOS: `.dmg`
- Linux: `.AppImage`

## Probar sin empaquetar
```
npm run prepare-app
npm start
```

## Actualizar a una versión nueva de la app
Borra `app-src/` (o deja que `prepare-app` haga `git pull`) y vuelve a ejecutar `npm run dist`.
