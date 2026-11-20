import { EmptyState, PageHeader, buttonClass } from "@unicloud/ui";
import { Link } from "react-router";

export function NotFoundPage() {
  return (
    <>
      <PageHeader kicker="404" title="Страницата не е намерена" />
      <EmptyState
        title="Няма такава страница"
        action={
          <Link to="/home" className={buttonClass("secondary")}>
            Към началото
          </Link>
        }
      >
        Проверете адреса или се върнете в началото.
      </EmptyState>
    </>
  );
}
