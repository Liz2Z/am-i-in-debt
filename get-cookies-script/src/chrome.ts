import CDP from "chrome-remote-interface";
import { spawn, execSync } from "child_process";

export interface ChromeOptions {
  port: number;
  profileName: string;
}

export function launchChrome(options: ChromeOptions): Promise<() => void> {
  return new Promise((resolve, reject) => {
    const { port, profileName } = options;
    const isMac = process.platform === "darwin";
    const isWindows = process.platform === "win32";
    const isLinux = process.platform === "linux";

    console.log(`🧹 清理端口 ${port}...`);

    if (isMac || isLinux) {
      try {
        try {
          const pid = execSync(`lsof -ti:${port}`, { encoding: "utf-8" }).trim();
          if (pid) {
            console.log(`🔪 杀死占用端口的进程: ${pid}`);
            execSync(`kill -9 ${pid}`);
          }
        } catch {
          // 没有进程占用端口，忽略
        }
      } catch {
        console.log("清理端口时出错，继续...");
      }
    }

    let chromePath: string;
    let args: string[];

    const tmpDir = process.env.TMPDIR || "/tmp";
    const userDataDir = `${tmpDir}/chrome-debug-${profileName}-${Date.now()}`;

    if (isMac) {
      chromePath = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
      args = [
        `--remote-debugging-port=${port}`,
        "--no-first-run",
        "--no-default-browser-check",
        `--user-data-dir=${userDataDir}`,
        "--no-sandbox",
        "--disable-background-networking",
        "--disable-backgrounding-occluded-windows",
        "--disable-renderer-backgrounding",
        "--disable-features=TranslateUI",
        "--disable-ipc-flooding-protection",
        "--disable-features=BlinkGenPropertyTrees",
      ];
    } else if (isWindows) {
      chromePath = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
      args = [
        `--remote-debugging-port=${port}`,
        "--no-first-run",
        "--no-default-browser-check",
        `--user-data-dir=${userDataDir}`,
        "--no-sandbox",
      ];
    } else {
      chromePath = "google-chrome";
      args = [
        `--remote-debugging-port=${port}`,
        "--no-first-run",
        "--no-default-browser-check",
        `--user-data-dir=${userDataDir}`,
        "--no-sandbox",
      ];
    }

    console.log(`🚀 启动 Chrome 浏览器...`);

    const chromeProcess = spawn(chromePath, args, {
      detached: true,
      stdio: "ignore",
    });

    chromeProcess.on("error", (err) => {
      reject(new Error(`启动 Chrome 失败: ${err.message}`));
    });

    chromeProcess.unref();

    let retries = 0;
    const maxRetries = 20;
    const checkInterval = setInterval(async () => {
      retries++;
      try {
        const testClient = await CDP({ port }).catch(() => null);
        if (testClient) {
          await testClient.close();
          clearInterval(checkInterval);
          console.log("✅ Chrome 已准备就绪");
          resolve(() => {
            try {
              if (isMac || isLinux) {
                spawn("pkill", ["-9", "-f", `chrome-debug-${profileName}`]);
              } else if (isWindows) {
                spawn("taskkill", ["/F", "/IM", "chrome.exe"]);
              }
            } catch {
              // 忽略关闭错误
            }
          });
        } else if (retries >= maxRetries) {
          clearInterval(checkInterval);
          reject(new Error("Chrome 启动超时"));
        }
      } catch {
        if (retries >= maxRetries) {
          clearInterval(checkInterval);
          reject(new Error("Chrome 启动超时"));
        }
      }
    }, 1000);
  });
}

export async function saveCookies(
  cookies: any[],
  outputDir: string
): Promise<void> {
  const relevantCookies = cookies;

  console.log("=".repeat(60));
  console.log("🎉 成功获取 Cookies:");
  console.log("=".repeat(60));

  relevantCookies.forEach((cookie) => {
    console.log(`\n📌 ${cookie.name}`);
    console.log(
      `   值: ${cookie.value.substring(0, 50)}${cookie.value.length > 50 ? "..." : ""}`
    );
    console.log(`   域名: ${cookie.domain}`);
    console.log(`   路径: ${cookie.path}`);
    console.log(`   安全: ${cookie.secure ? "是" : "否"}`);
    console.log(`   HttpOnly: ${cookie.httpOnly ? "是" : "否"}`);
  });

  const cookieString = relevantCookies.map((c) => `${c.name}=${c.value}`).join("; ");

  console.log("\n" + "=".repeat(60));
  console.log("📋 Cookie 字符串 (可直接用于请求头):");
  console.log("=".repeat(60));
  console.log(cookieString);

  const jsonPath = `${outputDir}/cookies.json`;
  const txtPath = `${outputDir}/cookies.txt`;

  await Bun.write(jsonPath, JSON.stringify(relevantCookies, null, 2));
  console.log(`\n💾 Cookies 已保存到 ${jsonPath}`);

  await Bun.write(txtPath, cookieString);
  console.log(`💾 Cookie 字符串已保存到 ${txtPath}`);
}
