import { QueryClient } from "@tanstack/react-query";

export function createAppQueryClient() {
  // 所有視窗透過本機 IPC 取得資料，不能因 WebView 的離線判定而暫停。
  // 外部 API 的連線錯誤由後端處理。
  return new QueryClient({
    defaultOptions: {
      queries: { networkMode: "always" },
      mutations: { networkMode: "always" },
    },
  });
}
