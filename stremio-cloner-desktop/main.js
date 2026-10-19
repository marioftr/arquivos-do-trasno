const { app, BrowserWindow, shell } = require("electron");
const { spawn } = require("node:child_process");
const net = require("node:net");
const path = require("node:path");

let server;

function freePort() {
  return new Promise((resolve, reject) => {
    const s = net.createServer();
    s.listen(0, "127.0.0.1", () => {
      const { port } = s.address();
      s.close(() => resolve(port));
    });
    s.on("error", reject);
  });
}

function waitForServer(port, tries = 100) {
  return new Promise((resolve, reject) => {
    const attempt = (n) => {
      const sock = net.connect(port, "127.0.0.1");
      sock.on("connect", () => { sock.destroy(); resolve(); });
      sock.on("error", () => {
        sock.destroy();
        if (n <= 0) reject(new Error("El servidor no arrancó"));
        else setTimeout(() => attempt(n - 1), 200);
      });
    };
    attempt(tries);
  });
}

async function createWindow() {
  const port = await freePort();
  const bundle = app.isPackaged
    ? path.join(process.resourcesPath, "bundle")
    : path.join(__dirname, "bundle");

  server = spawn(process.execPath, [path.join(bundle, "server.js")], {
    cwd: bundle,
    env: { ...process.env, ELECTRON_RUN_AS_NODE: "1", PORT: String(port), HOSTNAME: "127.0.0.1", NODE_ENV: "production" },
    stdio: "inherit",
  });

  await waitForServer(port);

  const win = new BrowserWindow({ width: 1200, height: 850, autoHideMenuBar: true });
  win.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url);
    return { action: "deny" };
  });
  win.loadURL(`http://127.0.0.1:${port}`);
}

app.whenReady().then(createWindow);
app.on("window-all-closed", () => app.quit());
app.on("quit", () => server && server.kill());
