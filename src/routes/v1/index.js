const express = require('express');
const satelliteRoute = require('./satellite.route');
const indexRoute = require('./default.route');

const router = express.Router();

const routes = [
  {
    path: '/',
    route: indexRoute,
  },
  {
    path: `/api/${process.env.API_VERSION}/satellite`,
    route: satelliteRoute,
  },
];

routes.forEach(({ path, route }) => {
  router.use(path, route);
});

module.exports = router;
