import { AbilityConstant, ConfigurationConstant, Want } from '@kit.AbilityKit';
import { hilog } from '@kit.PerformanceAnalysisKit';
import { window } from '@kit.ArkUI';
import { RustAbility } from '@ohos-rs/ability';

const DOMAIN = 0x0000;

export default class EntryAbility extends RustAbility {

  public moduleName: string = "mobile_demo";

  async onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): Promise<void> {
    super.onCreate(want, launchParam);
    this.context.getApplicationContext().setColorMode(ConfigurationConstant.ColorMode.COLOR_MODE_NOT_SET);
    hilog.info(DOMAIN, 'testTag', '%{public}s', 'Ability onCreate');
  }

  async onWindowStageCreate(windowStage: window.WindowStage): Promise<void> {
    let result = super.onWindowStageCreate(windowStage);
    const window = await windowStage.getMainWindow();
    window.setWindowSystemBarEnable([]);
    return result;
  }

}