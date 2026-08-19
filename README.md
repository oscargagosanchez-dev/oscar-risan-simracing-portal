# Oscar Risan SimRacing Portal

Base de producción del portal SimRacing de Oscar Risan.

## Despliegue seguro

1. `npm run preflight` debe terminar con `PREFLIGHT OK`.
2. Cada cambio se prueba primero en un Preview Deployment de Vercel.
3. Comprobar `/api/health`, `/api/events`, `/api/youtube`, noticias, imágenes y el modal `Ver detalles`.
4. Solo después se promociona el Preview validado a producción.
5. La versión pública estable anterior no se sustituye hasta completar todas las comprobaciones.

## Importar en Vercel

[![Deploy with Vercel](https://vercel.com/button)](https://vercel.com/new/clone?repository-url=https%3A%2F%2Fgithub.com%2Foscargagosanchez-dev%2Foscar-risan-simracing-portal&project-name=oscar-risan-simracing-portal)

Repositorio: `oscargagosanchez-dev/oscar-risan-simracing-portal`
