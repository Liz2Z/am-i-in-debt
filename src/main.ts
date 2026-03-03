import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface UsageInfo {
  total_tokens: number;
  used_tokens: number;
  remaining_tokens: number;
  reset_date: string;
  percentage: number;
}

// 格式化数字
function formatNumber(num: number): string {
  if (num >= 1000000) {
    return (num / 1000000).toFixed(1) + 'M';
  }
  if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'K';
  }
  return num.toString();
}

// 格式化日期
function formatDate(dateStr: string): string {
  if (!dateStr) return '未知';
  const date = new Date(dateStr);
  return `${date.getMonth() + 1}月${date.getDate()}日 重置`;
}

// 渲染加载状态
function renderLoading(): string {
  return `
    <div class="header">
      <div class="logo">Z</div>
      <div class="title">智谱 Coding Usage</div>
    </div>
    <div class="loading">加载中...</div>
  `;
}

// 渲染错误状态
function renderError(message: string): string {
  return `
    <div class="header">
      <div class="logo">Z</div>
      <div class="title">智谱 Coding Usage</div>
    </div>
    <div class="error">${message}</div>
    <button class="login-btn" onclick="refresh()">重试</button>
  `;
}

// 渲染未登录状态
function renderNoLogin(): string {
  return `
    <div class="header">
      <div class="logo">Z</div>
      <div class="title">智谱 Coding Usage</div>
    </div>
    <div class="no-login">
      <p>尚未登录智谱账号</p>
      <button class="login-btn" onclick="openLogin()">点击登录</button>
    </div>
  `;
}

// 渲染使用情况
function renderUsage(info: UsageInfo, refreshTime: string): string {
  const percentage = Math.min(info.percentage, 100);
  const barColor = percentage > 80 ? '#ff6b6b' : percentage > 50 ? '#ffa94d' : '';

  return `
    <div class="header">
      <div class="logo">Z</div>
      <div class="title">智谱 Coding Usage</div>
    </div>
    <div class="usage-info">
      <div class="progress-section">
        <div class="progress-label">
          <span>已使用 ${percentage.toFixed(1)}%</span>
          <span>${formatNumber(info.used_tokens)} / ${formatNumber(info.total_tokens)}</span>
        </div>
        <div class="progress-bar">
          <div class="progress-fill" style="width: ${percentage}%; ${barColor ? `background: ${barColor}` : ''}"></div>
        </div>
      </div>
      <div class="stats">
        <div class="stat-item">
          <div class="stat-value">${formatNumber(info.total_tokens)}</div>
          <div class="stat-label">总额度</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">${formatNumber(info.used_tokens)}</div>
          <div class="stat-label">已使用</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">${formatNumber(info.remaining_tokens)}</div>
          <div class="stat-label">剩余</div>
        </div>
      </div>
      <div class="reset-info">${formatDate(info.reset_date)}</div>
      <div class="refresh-time">更新于 ${refreshTime}</div>
    </div>
  `;
}

// 渲染登录页面
function renderLogin(): string {
  return `
    <div class="header">
      <div class="logo">Z</div>
      <div class="title">智谱 Coding Usage</div>
    </div>
    <div class="no-login">
      <p>正在打开浏览器...</p>
      <p style="font-size: 11px; opacity: 0.7; margin-top: 8px;">请在浏览器中完成登录，登录成功后此窗口将自动关闭</p>
    </div>
  `;
}

// 刷新数据
async function refresh() {
  const app = document.getElementById('app')!;
  app.innerHTML = renderLoading();

  try {
    const isLoggedIn = await invoke<boolean>('check_login_status');
    if (!isLoggedIn) {
      app.innerHTML = renderNoLogin();
      return;
    }

    const info: UsageInfo = await invoke('get_usage_info');
    const now = new Date();
    const timeStr = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;
    app.innerHTML = renderUsage(info, timeStr);
  } catch (error) {
    app.innerHTML = renderError(error as string);
  }
}

// 打开登录流程
async function openLogin() {
  const app = document.getElementById('app')!;
  app.innerHTML = renderLogin();

  // 打开 CDP 登录流程
  // 这里需要启动一个子进程来执行 get-cookies.ts
  // 由于 Tauri 的安全限制，我们需要使用 Tauri 的命令来执行
  try {
    // 使用 Node.js 子进程执行登录脚本
    const { spawn } = require('child_process');
    const proc = spawn('bun', ['run', 'get-cookies.ts'], {
      cwd: '/Users/zan/Craft/coding_plan_usage',
      stdio: 'inherit'
    });

    proc.on('close', async (code: number) => {
      if (code === 0) {
        // 登录成功，读取 cookies 并保存
        const fs = require('fs');
        const cookies = fs.readFileSync('/Users/zan/Craft/coding_plan_usage/cookies.json', 'utf-8');
        await invoke('save_cookies', { cookies });

        // 刷新数据
        await refresh();

        // 关闭窗口
        if (typeof window !== 'undefined' && window.__TAURI__) {
          const { getCurrentWindow } = await import('@tauri-apps/api/window');
          const win = getCurrentWindow();
          win.hide();
        }
      } else {
        app.innerHTML = renderError('登录失败，请重试');
      }
    });
  } catch (error) {
    app.innerHTML = renderError('启动登录流程失败: ' + error);
  }
}

// 全局函数暴露给 HTML
(window as any).refresh = refresh;
(window as any).openLogin = openLogin;

// 初始化
async function init() {
  await refresh();

  // 监听刷新事件
  await listen('refresh-usage', () => {
    refresh();
  });

  // 每30秒自动刷新
  setInterval(() => {
    refresh();
  }, 30000);
}

init();
