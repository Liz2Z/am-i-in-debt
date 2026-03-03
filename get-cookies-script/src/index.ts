/**
 * Chrome DevTools Protocol 获取智谱 Cookies
 *
 * 自动启动 Chrome 浏览器并获取 cookies
 * 使用环境变量 COOKIES_OUTPUT_PATH 指定输出目录
 */

import CDP from 'chrome-remote-interface';
import { spawn } from 'child_process';

const TARGET_URL = 'https://bigmodel.cn/usercenter/glm-coding/usage';
const CDP_PORT = 9222;

// 获取输出目录（从环境变量或当前目录）
const OUTPUT_DIR = process.env.COOKIES_OUTPUT_PATH || process.cwd();

// 启动 Chrome 浏览器
function launchChrome(): Promise<() => void> {
  return new Promise((resolve, reject) => {
    const isMac = process.platform === 'darwin';
    const isWindows = process.platform === 'win32';
    const isLinux = process.platform === 'linux';

    let chromePath: string;
    let args: string[];

    // 使用临时用户数据目录，避免与现有 Chrome 冲突
    const tmpDir = process.env.TMPDIR || '/tmp';
    const userDataDir = `${tmpDir}/chrome-debug-${Date.now()}`;

    if (isMac) {
      chromePath = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
      args = [
        `--remote-debugging-port=${CDP_PORT}`,
        '--no-first-run',
        '--no-default-browser-check',
        `--user-data-dir=${userDataDir}`,
        '--no-sandbox',
        '--disable-background-networking',
        '--disable-backgrounding-occluded-windows',
        '--disable-renderer-backgrounding',
        '--disable-features=TranslateUI',
        '--disable-ipc-flooding-protection',
        '--disable-features=BlinkGenPropertyTrees'
      ];
    } else if (isWindows) {
      chromePath = 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe';
      args = [
        `--remote-debugging-port=${CDP_PORT}`,
        '--no-first-run',
        '--no-default-browser-check',
        `--user-data-dir=${userDataDir}`,
        '--no-sandbox'
      ];
    } else {
      // Linux
      chromePath = 'google-chrome';
      args = [
        `--remote-debugging-port=${CDP_PORT}`,
        '--no-first-run',
        '--no-default-browser-check',
        `--user-data-dir=${userDataDir}`,
        '--no-sandbox'
      ];
    }

    console.log(`🚀 启动 Chrome 浏览器...`);

    const chromeProcess = spawn(chromePath, args, {
      detached: true,
      stdio: 'ignore'
    });

    chromeProcess.on('error', (err) => {
      reject(new Error(`启动 Chrome 失败: ${err.message}`));
    });

    // Chrome 启动后分离进程
    chromeProcess.unref();

    // 等待 Chrome 启动并创建可检查的 target
    // 增加等待时间并定期检查
    let retries = 0;
    const maxRetries = 10;
    const checkInterval = setInterval(async () => {
      retries++;
      try {
        // 尝试连接 CDP 来检查 Chrome 是否准备好
        const testClient = await CDP({ port: CDP_PORT }).catch(() => null);
        if (testClient) {
          await testClient.close();
          clearInterval(checkInterval);
          console.log('✅ Chrome 已准备就绪');
          resolve(() => {
            // 清理函数：关闭这个特定的 Chrome 实例
            try {
              if (isMac || isLinux) {
                spawn('pkill', ['-9', '-f', 'chrome-debug']);
              } else if (isWindows) {
                spawn('taskkill', ['/F', '/IM', 'chrome.exe']);
              }
            } catch (e) {
              // 忽略关闭错误
            }
          });
        } else if (retries >= maxRetries) {
          clearInterval(checkInterval);
          reject(new Error('Chrome 启动超时'));
        }
      } catch (e) {
        if (retries >= maxRetries) {
          clearInterval(checkInterval);
          reject(new Error('Chrome 启动超时'));
        }
      }
    }, 1000);
  });
}

async function main() {
  let client: Awaited<ReturnType<typeof CDP>> | null = null;
  let cleanupChrome: (() => void) | null = null;

  try {
    // 启动 Chrome
    cleanupChrome = await launchChrome();

    console.log('🔌 连接到 Chrome DevTools Protocol...');
    client = await CDP({ port: CDP_PORT });

    const { Network, Page, Runtime, Target } = client;

    // 启用必要的域
    await Network.enable();
    await Page.enable();
    await Runtime.enable();

    console.log('📄 打开智谱 coding usage 页面...');
    await Page.navigate({ url: TARGET_URL });

    console.log('⏳ 等待页面加载...');
    await Page.loadEventFired();

    console.log('\n' + '='.repeat(60));
    console.log('✅ 浏览器已打开页面，请在浏览器中完成登录');
    console.log('🔍 登录成功后，脚本会自动检测并获取 Cookies');
    console.log('='.repeat(60) + '\n');

    // 轮询检查登录状态
    let isLoggedIn = false;
    let attempts = 0;
    const maxAttempts = 300; // 最多等待 5 分钟

    while (!isLoggedIn && attempts < maxAttempts) {
      attempts++;

      try {
        // 检查是否跳转到了 usage 页面（登录成功的标志）
        const urlResult = await Runtime.evaluate({
          expression: 'window.location.href',
          returnByValue: true
        });

        const currentUrl = urlResult.result.value;
        console.log(`[${attempts}] 当前页面: ${currentUrl}`);

        // 如果 URL 包含 usage 并且不是在登录页面，说明登录成功
        if (currentUrl.includes('glm-coding/usage')) {
          // 额外检查是否有用户信息元素
          const userCheck = await Runtime.evaluate({
            expression: `
              document.querySelector('[class*="user"]') ||
              document.querySelector('[class*="avatar"]') ||
              document.cookie.includes('authorization')
            `,
            returnByValue: true
          });

          if (userCheck.result.value) {
            isLoggedIn = true;
            break;
          }
        }

        // 检查是否被重定向到登录页
        if (currentUrl.includes('login')) {
          console.log('⚠️  检测到登录页面，等待用户完成登录...');
        }

      } catch (error) {
        // 页面可能还在加载，忽略错误继续轮询
      }

      // 每秒检查一次
      await new Promise(resolve => setTimeout(resolve, 1000));
    }

    if (!isLoggedIn) {
      console.log('\n❌ 登录检测超时，请重试');
      process.exit(1);
    }

    console.log('\n✅ 检测到登录成功！');
    console.log('🍪 正在获取 Cookies...\n');

    // 获取所有 cookies
    const cookiesResult = await Network.getCookies();

    // 筛选 bigmodel.cn 相关的 cookies
    const relevantCookies = cookiesResult.cookies.filter(cookie =>
      cookie.domain.includes('bigmodel') ||
      cookie.domain.includes('zhipuai')
    );

    console.log('='.repeat(60));
    console.log('🎉 成功获取 Cookies:');
    console.log('='.repeat(60));

    // 打印每个 cookie
    relevantCookies.forEach(cookie => {
      console.log(`\n📌 ${cookie.name}`);
      console.log(`   值: ${cookie.value.substring(0, 50)}${cookie.value.length > 50 ? '...' : ''}`);
      console.log(`   域名: ${cookie.domain}`);
      console.log(`   路径: ${cookie.path}`);
      console.log(`   安全: ${cookie.secure ? '是' : '否'}`);
      console.log(`   HttpOnly: ${cookie.httpOnly ? '是' : '否'}`);
    });

    // 生成 Cookie 字符串（用于 API 请求）
    const cookieString = relevantCookies
      .map(c => `${c.name}=${c.value}`)
      .join('; ');

    console.log('\n' + '='.repeat(60));
    console.log('📋 Cookie 字符串 (可直接用于请求头):');
    console.log('='.repeat(60));
    console.log(cookieString);

    // 保存到文件（使用指定输出目录）
    const jsonPath = `${OUTPUT_DIR}/cookies.json`;
    const txtPath = `${OUTPUT_DIR}/cookies.txt`;

    await Bun.write(jsonPath, JSON.stringify(relevantCookies, null, 2));
    console.log(`\n💾 Cookies 已保存到 ${jsonPath}`);

    await Bun.write(txtPath, cookieString);
    console.log(`💾 Cookie 字符串已保存到 ${txtPath}`);

  } catch (error) {
    console.error('❌ 发生错误:', error);
    process.exit(1);
  } finally {
    if (client) {
      await client.close();
    }
    if (cleanupChrome) {
      console.log('🧹 关闭 Chrome 浏览器...');
      cleanupChrome();
    }
  }
}

main();
