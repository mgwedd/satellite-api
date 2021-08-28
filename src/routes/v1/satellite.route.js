/* eslint-disable prettier/prettier */
const express = require('express');

const { satelliteController } = require('../../controllers');

const router = express.Router();

router
  .route('/')
  .get(satelliteController.listSatellites)
  .post(satelliteController.bulkCreateSatellites);

router
  .route('/:id')
  .get(satelliteController.getSatellite)
  .patch(satelliteController.updateSatellite)
  .delete(satelliteController.deleteSatellite);

router
  .route('/:id/next-visible')
  .get(satelliteController.getNextVisible)

router
  .route('/overhead')
  .get(satelliteController.getOverhead)


module.exports = router;
