# البدء السريع مع cargo-target-gc

cargo-target-gc هو جامع مخلفات لآثار البناء الخاصة بـ Cargo داخل `target/`.
يفحص مجلد `target/` في المشروع أو workspace ويعرض المساحة التي يمكن استعادتها.
لا يحذف آثار البناء القديمة التي يعتبرها آمنة إلا بعد تأكيد صريح.

## لماذا توجد هذه الأداة

مجلدات Cargo `target/` كانت تكبر مع الوقت دائما، لكن vibe coding و agentic
coding يجعلان النمو أسرع وأسهل في أن يمر من دون ملاحظة. أدوات مثل Claude Code
و Codex و Gemini CLI وغيرها من وكلاء البرمجة قد تشغل build و test و retry وتبدل
المهام مرات كثيرة في جلسة واحدة. cargo-target-gc يوفر مسارا محافظا للتنظيف:
scan أولا، ثم معاينة باستخدام `--dry-run`، ثم الحذف فقط بعد تأكيد صريح.

## مكان التشغيل

شغله في نفس مجلد مشروع Cargo أو workspace الذي تشغل منه `cargo build`.

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

إذا كان wrapper مثل `make` يبني مشروع Cargo داخليا، فانتقل أولا إلى مجلد Cargo
ذلك ثم شغل `cargo target-gc` هناك. الأداة لا تخمن مسارات بناء مخفية خاصة
بالـ wrappers.

## الأوامر الأساسية

```bash
cargo target-gc scan
cargo target-gc scan --json
cargo target-gc clean --dry-run
cargo target-gc clean --dry-run --stale
cargo target-gc clean --dry-run --profile-cache
cargo target-gc clean --confirm --stale
cargo target-gc config
cargo target-gc install-agent-skills
```

## قواعد الأمان

- `scan` للقراءة فقط ولا يشغل Cargo أبدا.
- `clean` يرفض العمل من دون خيار واحد فقط من `--dry-run` أو `--confirm`.
- افتراضيا، يستعيد `clean` ذاكرة incremental القديمة فقط.
- مع `--stale` يستعيد أيضا الآثار stale الأقدم من مدة الاحتفاظ.
- `--profile-cache` وضع أقوى ويشمل أيضا incremental cache حديثة ومجلدات حديثة
  مثل `deps` و `build` و `.fingerprint` و `examples`. افحص أولا باستخدام
  `--dry-run`.
- `cargo clean` بدون خيارات يحذف target كله، وخيارات Cargo مثل `--package`
  و `--profile` و `--target` تنظف النطاق المحدد كله. target-gc ينظف حسب العمر
  والفئة للحفاظ على قدر أكبر من build cache.
- إذا بدا أن عملية Cargo/rustc نشطة تستخدم target المختار، يتم رفض الحذف المؤكد.
- مسارات الحذف تبقى محصورة داخل جذر Cargo `target/` تم التحقق منه.

## الإعداد

يضبط `target-gc.toml` في جذر المشروع سياسة الاحتفاظ.

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
