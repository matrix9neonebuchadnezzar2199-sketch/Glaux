//! プロンプト支援メニュー（公刊資料の整理・調査向けテンプレート）

use egui::{popup, Button, Color32, CornerRadius, Frame, Id, Margin, RichText, Stroke, Ui, Vec2};

/// ライト/ダーク共通の紺色メニュー（テーマ非連動）
const MENU_BG: Color32 = Color32::from_rgb(0x1B, 0x2B, 0x4A);
const MENU_ITEM_HOVER: Color32 = Color32::from_rgb(0x2A, 0x3F, 0x6B);
const MENU_BORDER: Color32 = Color32::from_rgb(0x3D, 0x52, 0x7A);
const MENU_TEXT: Color32 = Color32::WHITE;

const APPLY_TEMP_ID: &str = "glaux_prompt_assist_apply";

struct PromptAssistItem {
    label: &'static str,
    body: &'static str,
}

const ITEMS: &[PromptAssistItem] = &[
    PromptAssistItem {
        label: "要約",
        body: "次の文章を要約してください。\n\n【ルール】\n- 日本語で、100文字以内（句読点含む）\n- 事実のみ。推測・評価・感想は入れない\n- 固有名詞・数値・日付はできるだけ残す\n- 1文で完結させる（箇条書き不可）\n\n【対象文章】\n※ここに要約したい文章を貼りつけ※",
    },
    PromptAssistItem {
        label: "翻訳",
        body: "次の文章を日本語に翻訳してください。\n\n【ルール】\n- 意味を変えず、原文の情報量を維持する\n- 公文体（だ・である調）で統一\n- 専門用語・固有名詞は初出時に原文を（）で併記\n- 原文にない補足説明はしない\n- 不明な語は「［訳不明: 原文］」とする\n\n【対象文章】\n※ここに翻訳したい文章を貼りつけ※",
    },
    PromptAssistItem {
        label: "WIKI",
        body: "次の単語・用語について、知っている範囲で説明してください。\n\n【対象】\n※ここに調べたい単語・用語を入力※\n\n【ルール】\n- 日本語で回答\n- 以下の見出しで整理する（該当なしは「不明」と書く）\n  1. 定義（一言）\n  2. 概要（2〜4文）\n  3. 分野・文脈（どの領域で使われるか）\n  4. 関連語・類語\n  5. 注意（曖昧な点・複数の意味がある場合）\n- 確信が低い場合は「一般知識に基づく推定」と明記する\n- 出典・URLは書かない（オフラインのため未検証）",
    },
    PromptAssistItem {
        label: "人名・地名抽出",
        body: "次の文章から、人名・地名・組織名・技術的な専門用語を抽出してください。\n\n【ルール】\n- 日本語で回答\n- カテゴリ別に表形式で出力する\n  | カテゴリ | 名称 | 文中での役割（短く） |\n- カテゴリ: 人名 / 地名 / 組織・団体 / 技術用語・略語 / その他固有名詞\n- 同じ名称の重複は1行にまとめる\n- 推測で補完しない。文中に現れないものは列挙しない\n- 略語は初出時に展開できる場合のみ（）で補足\n\n【対象文章】\n※ここに対象文章を貼りつけ※",
    },
    PromptAssistItem {
        label: "構造化要約",
        body: "次の文章を構造化して要約してください。\n\n【ルール】\n- 日本語で回答\n- 以下の見出しをこの順で埋める（該当なしは「なし」）\n  ## テーマ（1行）\n  ## 要点（3〜5項目、箇条書き）\n  ## 数値・日付・固有名詞\n  ## 結論・示唆\n  ## 本文に書かれていないこと（推測はここに分離）\n- 原文にない情報は書かない\n\n【対象文章】\n※ここに要約したい文章を貼りつけ※",
    },
    PromptAssistItem {
        label: "時系列整理",
        body: "次の文章に含まれる出来事を時系列で整理してください。\n\n【ルール】\n- 日本語で回答\n- 表形式: | 日付（または時期） | 出来事 | 関係する主体 |\n- 日付が不明確なものは「時期不明」とし、文中の表現をそのまま書く\n- 文中にない日付を推測で補完しない\n\n【対象文章】\n※ここに対象文章を貼りつけ※",
    },
    PromptAssistItem {
        label: "用語集",
        body: "次の文章に含まれる専門用語・略語の用語集を作成してください。\n\n【ルール】\n- 日本語で回答\n- 表形式: | 用語 | 定義（文中の意味に即して） | 備考 |\n- 文中に現れる用語のみ。推測で追加しない\n- 略語は可能なら展開し、不確かなら「不明」とする\n\n【対象文章】\n※ここに対象文章を貼りつけ※",
    },
    PromptAssistItem {
        label: "論点整理",
        body: "次の文章の論点を整理してください。\n\n【ルール】\n- 日本語で回答\n- 以下の見出しで整理\n  ## 主張・結論\n  ## 根拠・データ（文中に書かれたもののみ）\n  ## 前提・仮定\n  ## 未記載・不明な点\n  ## 検証が必要な点（推測はここに分離）\n- 原文にない評価は「推測」と明記\n\n【対象文章】\n※ここに対象文章を貼りつけ※",
    },
    PromptAssistItem {
        label: "箇条書き化",
        body: "次の文章を階層付き箇条書きに整理してください。\n\n【ルール】\n- 日本語で回答\n- 階層は最大3段（・ / - / 　-）\n- 原文の情報を落とさない。要約しすぎない\n- 数値・固有名詞・日付はそのまま残す\n\n【対象文章】\n※ここに対象文章を貼りつけ※",
    },
    PromptAssistItem {
        label: "対照",
        body: "次の2つの文章を対照してください。\n\n【ルール】\n- 日本語で回答\n- 以下の見出しで整理\n  ## 共通点\n  ## 相違点（表形式推奨: | 項目 | 文章A | 文章B |）\n  ## 文章Aのみに含まれる情報\n  ## 文章Bのみに含まれる情報\n- 原文にない解釈は「推測」と明記\n\n【文章A】\n※ここに1つ目の文章を貼りつけ※\n\n【文章B】\n※ここに2つ目の文章を貼りつけ※",
    },
];

/// 「プロンプト支援」ボタンとクリックで開くメニューを描画する
pub fn draw_prompt_assist(ui: &mut Ui) {
    let response = ui.add(navy_button("プロンプト支援", Vec2::new(120.0, 28.0)));

    if response.clicked() {
        ui.memory_mut(|mem| mem.toggle_popup(response.id));
    }

    popup::popup_below_widget(
        ui,
        response.id,
        &response,
        popup::PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            Frame::new()
                .fill(MENU_BG)
                .stroke(Stroke::new(1.0, MENU_BORDER))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    ui.set_min_width(220.0);
                    ui.visuals_mut().override_text_color = Some(MENU_TEXT);
                    for item in ITEMS {
                        if navy_menu_item(ui, item.label).clicked() {
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(Id::new(APPLY_TEMP_ID), item.body.to_string());
                            });
                            ui.memory_mut(|mem| mem.close_popup());
                        }
                    }
                });
        },
    );
}

/// メニュー選択後、同一フレーム内で呼び出して入力欄へ反映する
pub fn take_applied_template(ctx: &egui::Context) -> Option<String> {
    ctx.data_mut(|d| d.remove_temp::<String>(Id::new(APPLY_TEMP_ID)))
}

fn navy_button(label: &str, min_size: Vec2) -> Button<'_> {
    Button::new(RichText::new(label).color(MENU_TEXT))
        .fill(MENU_BG)
        .stroke(Stroke::new(1.0, MENU_BORDER))
        .corner_radius(CornerRadius::same(6))
        .min_size(min_size)
}

fn navy_menu_item(ui: &mut Ui, label: &str) -> egui::Response {
    let size = Vec2::new(ui.available_width().max(220.0), 30.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let fill = if response.hovered() {
        MENU_ITEM_HOVER
    } else {
        Color32::TRANSPARENT
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(4), fill);
    ui.painter().text(
        rect.left_center() + Vec2::new(10.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(ui.style().text_styles[&egui::TextStyle::Body].size),
        MENU_TEXT,
    );
    response
}
