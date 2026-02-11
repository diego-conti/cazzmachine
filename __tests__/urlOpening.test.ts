import { openUrl } from "./lib/tauri";

describe("URL Opening", () => {
  it("should open URLs correctly on Android", async () => {
    const mockOpen = jest.fn();
    jest.mock("@tauri-apps/plugin-shell", () => ({
      open: mockOpen,
    }));

    await openUrl("https://example.com");
    expect(mockOpen).toHaveBeenCalledWith("https://example.com");
  });

  it("should handle Android-specific URL opening", async () => {
    (global as any).window = {
      Tauri: {
        invoke: jest.fn().mockResolvedValue(undefined),
      },
    };

    await openUrl("https://example.com");
    expect((global as any).window.Tauri.invoke).toHaveBeenCalledWith("open_url", { url: "https://example.com" });
  });
});