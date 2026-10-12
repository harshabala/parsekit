import { checkForUpdate, installUpdate, type UpdateInfo } from "./update";
import { t } from "./i18n.svelte";

export class UpdateState {
  available = $state<UpdateInfo | null>(null);
  isInstalling = $state(false);
  error = $state<string | null>(null);
  checkBusy = $state(false);
  statusNote = $state<string | null>(null);
  statusOk = $state(false);

  scheduleBackgroundCheck(_appVersion: string) {
    void checkForUpdate()
      .then((info) => {
        if (info.available) {
          this.available = info;
        }
      })
      .catch(() => {
        /* silent — offline or misconfigured endpoint */
      });
  }

  async checkForUpdates(appVersion: string) {
    this.statusNote = null;
    this.statusOk = false;
    this.error = null;
    this.checkBusy = true;
    try {
      const info = await checkForUpdate();
      if (info.available) {
        this.available = info;
        this.statusNote = null;
      } else {
        this.statusNote = t("update.upToDate", { version: appVersion });
        this.statusOk = true;
      }
    } catch {
      this.statusNote = t("update.checkFailed");
    } finally {
      this.checkBusy = false;
    }
  }

  async installAvailable() {
    this.isInstalling = true;
    this.error = null;
    try {
      await installUpdate();
    } catch (e) {
      this.error = (e instanceof Error ? e.message : String(e)) || t("update.installFailed");
      this.isInstalling = false;
    }
  }

  dismiss() {
    this.available = null;
    this.error = null;
  }
}

export const updateState = new UpdateState();
