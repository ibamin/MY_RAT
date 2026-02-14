const { contextBridge } = require('electron');

contextBridge.exposeInMainWorld('appMeta', {
  name: 'Red Team Simulation',
});
