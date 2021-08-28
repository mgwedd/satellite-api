/* eslint-disable prettier/prettier */
const express = require('express');

const { satteliteController } = require('../../controllers');

const router = express.Router();

router
  .route('/')
  .get(satteliteController.listSattelites)
  .post(satteliteController.bulkCreateSattelites);

router
  .route('/:id')
  .get(satteliteController.getSattelite)
  .patch(satteliteController.updateSattelite)
  .delete(satteliteController.deleteSattelite);

router
  .route('/overhead')
  .get(satteliteController.getOverhead)

router
  .route('/next-visible/:id')
  .get(satteliteController.getNextVisible)

module.exports = router;
