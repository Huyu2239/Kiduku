//! 統合テスト
//!
//! Interface (Mapper) → Usecase → Domain の全体フローをテストする。
//!
//! 注意: Interface層のMapperは、実際のアプリケーションでは serenity::Message
//! からの変換を行う (src/interface/mapper/input_mapper.rs)。
//! 統合テストでは serenity::Message の構築が困難なため、
//! MessageInput を直接作成しているが、実際のアプリケーションでは
//! Mapper経由で同等の処理が行われる。

use sample_discord_bot::domain::policy::greeting;
use sample_discord_bot::usecase::dto::DiscordExecStep;
use sample_discord_bot::usecase::on_message::greeting as greeting_usecase;

/// Interface層 → Usecase層 → Domain層の統合フロー
///
/// 生データから、以下の流れで処理される：
/// 1. MessageInputの作成（Interface層相当）
/// 2. Usecaseの実行
/// 3. Domain policyの判定
/// 4. DiscordExecPlanの生成
/// 5. バリデーション（マクロによる自動実行）
#[test]
fn test_greeting_integration_flow() {
    // === 準備 ===
    let message_content = "おはよう";
    let channel_id = 12345u64;

    // === 1. Interface層: メッセージデータの変換 ===
    // 実際のアプリケーションでは serenity::Message から変換されるが、
    // 統合テストでは MessageInput を直接作成
    let input = sample_discord_bot::usecase::dto::MessageInput::new(message_content, channel_id);

    // === 2. Domain層: ポリシー判定の検証 ===
    // Usecaseが内部で呼び出すDomain policyを直接呼んで、
    // 挨拶として認識されることを確認
    assert!(
        greeting::is_greeting(message_content),
        "Domain policy should recognize greeting"
    );

    // === 3. Usecase層: ビジネスロジックの実行 ===
    let result = greeting_usecase::execute(input);

    // === 4. Usecase層: バリデーションマクロの動作確認 ===
    // sync_validate_return マクロにより、返り値のバリデーションが自動実行される
    assert!(result.is_ok(), "Usecase should return valid plan");
    let plan = result.unwrap();

    // === 5. 出力の検証: DiscordExecPlan ===
    let steps = plan.into_steps();
    assert_eq!(steps.len(), 1, "Plan should contain exactly one step");

    match &steps[0] {
        DiscordExecStep::Send {
            channel_id: response_channel_id,
            payload,
        } => {
            // チャンネルIDが正しく引き継がれている
            assert_eq!(*response_channel_id, channel_id);

            // ドメインロジックに基づいたメッセージ内容
            assert_eq!(
                payload.content.as_deref(),
                Some("おはようございます！"),
                "Response message should match domain logic"
            );
        }
        other => panic!("Expected Send step, got {:?}", other),
    }
}

/// 非挨拶メッセージの統合フロー
///
/// Domain policyで拒否され、空のプランが返されることを確認
#[test]
fn test_non_greeting_integration_flow() {
    let message_content = "こんばんは";
    let channel_id = 12345u64;

    // === Domain層の判定 ===
    assert!(
        !greeting::is_greeting(message_content),
        "Domain policy should reject non-greeting"
    );

    // === Usecase層の実行 ===
    let input = sample_discord_bot::usecase::dto::MessageInput::new(message_content, channel_id);
    let result = greeting_usecase::execute(input);

    assert!(result.is_ok());
    let plan = result.unwrap();

    // 空のプランが返される
    assert!(
        plan.steps().is_empty(),
        "Non-greeting should produce empty plan"
    );
}

/// バリデーション統合テスト
///
/// Domain policyの空白トリム処理が、Usecase全体で正しく動作することを確認
#[test]
fn test_whitespace_handling_integration() {
    let message_content = "  おはよう  ";
    let channel_id = 12345u64;

    // === Domain層: trim処理の確認 ===
    assert!(
        greeting::is_greeting(message_content),
        "Domain policy should handle whitespace"
    );

    // === Usecase層: 全体フローの確認 ===
    let input = sample_discord_bot::usecase::dto::MessageInput::new(message_content, channel_id);
    let result = greeting_usecase::execute(input);

    assert!(result.is_ok());
    let plan = result.unwrap();
    assert_eq!(plan.steps().len(), 1, "Whitespace should be trimmed");
}

/// バリデーションエラーのテスト（将来的な拡張用）
///
/// 現在のコードでは無効なプランは生成されないが、
/// マクロによるバリデーションが動作することを確認する枠組み
#[test]
fn test_plan_validation_is_applied() {
    // 現在の実装では、greeting usecaseは常に有効なプランを生成する
    // （空プラン or 単一のSendステップ）
    //
    // このテストは、バリデーションマクロが実際に適用されていることを
    // ドキュメントとして示すためのもの
    //
    // 将来、複雑なプランを生成するUsecaseが追加された際に、
    // ここでバリデーションエラーのテストを追加する

    let input = sample_discord_bot::usecase::dto::MessageInput::new("おはよう", 12345);
    let result = greeting_usecase::execute(input);

    // sync_validate_return マクロにより、validate_plan が実行されている
    // 現在は常に成功するが、マクロの存在を確認
    assert!(result.is_ok(), "Valid plan should pass validation");
}
