import { useQuery } from "@tanstack/react-query";
import { CourseCard, EmptyState, PageHeader, buttonClass } from "@unicloud/ui";
import { Link } from "react-router";
import { api } from "../api/client";
import { keys } from "../api/hooks";
import { QueryState } from "../components/QueryState";

export function TeachPage() {
  const query = useQuery({ queryKey: keys.teach, queryFn: api.teachOfferings });
  return (
    <>
      <PageHeader kicker="Преподавател" title="Моите курсове" lede="Отворете курс, за да видите записаните студенти." />
      <QueryState query={query}>
        {(list) =>
          list.length === 0 ? (
            <EmptyState title="Нямате курсове този семестър" />
          ) : (
            <div className="uc-grid">
              {list.map((o) => (
                <CourseCard
                  key={o.id}
                  code={o.code}
                  title={o.title}
                  status={o.is_open ? "open" : "closed"}
                  meta={[o.semester]}
                  seats={{ taken: o.enrolled, capacity: o.capacity }}
                  actions={
                    <Link to={`/teach/offerings/${o.id}`} className={buttonClass("secondary", "sm")}>
                      Студенти
                    </Link>
                  }
                />
              ))}
            </div>
          )
        }
      </QueryState>
    </>
  );
}
