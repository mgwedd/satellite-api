const express = require('express');
const satelliteRoute = require('./satellite.route');
const indexRoute = require('./v1/default.route');

const router = express.Router();

const routes = [
  {
    path: '/',
    route: indexRoute,
  },
  {
    path: '/api',
    route: satelliteRoute,
  },
];

routes.forEach(({ path, route }) => {
  router.use(path, route);
});

module.exports = router;
